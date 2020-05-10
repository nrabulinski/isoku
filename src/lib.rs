#![feature(async_closure, const_generics)]
use futures::{future::join_all, stream::TryStreamExt};
use hyper::{Body, Request, Response};
use sqlx::{postgres::PgPool, prelude::*};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, instrument, trace};

pub use isoku_macros::event_data;

#[macro_use]
pub mod packets;
use packets::server as p;
pub mod token;
pub use token::Token;
pub mod channel;
pub use channel::Channel;
pub mod bot;
pub mod events;
pub mod r#match;
pub use r#match::Match;

const PROTOCOL_VERSION: u32 = 19;

#[derive(Debug)]
pub struct Glob {
    pub db_pool: PgPool,
    pub token_list: RwLock<HashMap<String, Arc<Token>>>,
    pub channel_list: RwLock<HashMap<String, Arc<Channel>>>,
    pub match_list: RwLock<HashMap<u16, Arc<Match>>>,
    pub lobby: RwLock<Vec<Arc<Token>>>,
    pub bot: Arc<Token>,
}

impl Glob {
    pub async fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let db_pool = PgPool::new(&db_url).await.unwrap();
        let mut channel_list = HashMap::with_capacity(2);
        Channel::new(&mut channel_list, "#osu", "wojexe to ciota", true);
        Channel::new(&mut channel_list, "#lobby", "multi", true);
        let mut token_list = HashMap::with_capacity(2);
        let bot = Token::new(&mut token_list, 3, "kenshichi".to_string());
        for ch in channel_list.values() {
            ch.user_join(bot.clone()).await;
        }
        let token_list = RwLock::new(token_list);
        let channel_list = RwLock::new(channel_list);
        let match_list = RwLock::default();
        let lobby = RwLock::default();
        Glob {
            db_pool,
            token_list,
            channel_list,
            match_list,
            lobby,
            bot,
        }
    }
}

#[instrument(skip(glob))]
async fn login(body: &[u8], glob: Arc<Glob>) -> Result<(String, Vec<u8>), &'static str> {
    let (username, password) = {
        let login_data = std::str::from_utf8(body).map_err(|_| "Bad request")?;
        let mut login_data = login_data.split('\n');
        let username = login_data.next().ok_or("Bad request")?;
        let password = login_data.next().ok_or("Bad request")?;
        (username.trim(), password)
    };
    if username == "wojexe" {
        return Err("wojexe to ciota");
    }
    let id: Option<(i32,)> =
        sqlx::query_as("SELECT id FROM users WHERE username = $1 AND password = $2")
            .bind(username)
            .bind(password)
            .fetch_optional(&glob.db_pool)
            .await
            .unwrap();
    let id = match id {
        Some((id,)) => id,
        None => return Err("User doesn't exist"),
    };
    if glob.token_list.read().await.values().any(|t| t.id == id) {
        return Err("Already logged in?");
    }
    let token = {
        let mut list = glob.token_list.write().await;
        Token::new(&mut list, id, username.to_string())
    };
    let online: Vec<i32> = glob
        .token_list
        .read()
        .await
        .values()
        .map(|t| t.id)
        .collect();
    let channels = join_all(
        glob.channel_list
            .read()
            .await
            .values()
            .filter(|ch| ch.public)
            .map(|ch| p::channel_info(ch)),
    )
    .await;
    let token_list = glob
        .token_list
        .read()
        .await
        .values()
        .flat_map(|t| p::user_panel(t))
        .collect();
    let user_stats = p::user_stats(&token).await;
    let data = [
        p::silence_end(0),
        p::protocol_ver(PROTOCOL_VERSION),
        p::user_id(token.id),
        p::user_rank(0),
        p::friend_list(&[]),
        p::user_panel(&token),
        user_stats,
        p::online_users(&online),
        token_list,
        p::channel_info_end(),
        channels.into_iter().flatten().collect(),
    ]
    .concat();
    Ok((token.token.clone(), data))
}

#[instrument(skip(glob, token))]
async fn handle_packet(
    mut body: &[u8],
    glob: Arc<Glob>,
    token: &str,
) -> Result<(String, Vec<u8>), &'static str> {
    let token = match glob.token_list.read().await.get(token) {
        Some(t) => t.clone(),
        None => return Err("Wrong token"),
    };
    trace!("handling packet");
    let mut res = Vec::new();
    use packets::Id;
    while body.len() >= 7 {
        let (id, len) = packets::parse_packet(body).map_err(|_| "Couldn't parse packet")?;
        let (data, rest) = (&body[7..]).split_at(len);
        body = rest;
        let packet = match id {
            Id::Unknown => {
                println!("UNKNOWN ID!");
                continue;
            }
            Id::SendPublicMessage => {
                if let Err(e) = events::send_message::public(data, &token, &glob).await {
                    Err(e)
                } else {
                    continue;
                }
            }
            Id::SendPrivateMessage => {
                if let Err(e) = events::send_message::private(data, &token, &glob).await {
                    Err(e)
                } else {
                    continue;
                }
            }
            Id::Pong => continue,
            Id::UserStatsRequest => events::stats_request::handle(data, &glob).await,
            Id::RequestStatusUpdate => events::status_update::handle(&token).await,
            Id::CreateMatch => events::matches::create(data, &token, &glob).await,
            Id::MatchChangeSettings => {
                if let Err(e) = events::matches::change_settings(data, &token, &glob).await {
                    Err(e)
                } else {
                    continue;
                }
            }
            Id::JoinLobby => events::lobby_join::handle(&token, &glob).await,
            Id::PartLobby => {
                if let Err(e) = events::lobby_part::handle(&token, &glob).await {
                    Err(e)
                } else {
                    continue;
                }
            }
            Id::ChannelJoin => events::channel_join::handle(data, &token, &glob).await,
            Id::ChannelPart => events::channel_part::handle(data, &token, &glob).await,
            Id::ChangeAction => {
                if let Err(e) = events::change_action::handle(data, &token, &glob).await {
                    Err(e)
                } else {
                    continue;
                }
            }
            Id::Logout => {
                events::logout::handle(&token.token, &glob).await?;
                break;
            }
            _ => Err(format!("Unhandled packet {:?}", id)),
        };
        match packet {
            Ok(mut d) => res.append(&mut d),
            Err(msg) => res.append(&mut p::notification(&msg)),
        }
    }
    let mut queue = token.queue.lock().await;
    res.append(&mut queue);
    Ok((token.token.clone(), res))
}

pub async fn main_handler(req: Request<Body>, glob: Arc<Glob>) -> http::Result<Response<Body>> {
    let (parts, body) = req.into_parts();
    let body: Vec<u8> = body
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
        .unwrap();
    let res = match parts.headers.get("osu-token") {
        None => login(&body, glob).await,
        Some(token) => {
            let token = match token.to_str() {
                Ok(token) => token,
                Err(e) => {
                    error!(?e);
                    return Response::builder().status(400).body(Body::empty());
                }
            };
            handle_packet(&body, glob, token).await
        }
    };
    let (token, data) = res.unwrap_or_else(|e| {
        (
            "0".to_string(),
            [p::notification(&e), p::login_failed()].concat(),
        )
    });
    trace!(?token, ?data);
    Response::builder()
        .header("cho-token", &token)
        .header("cho-protocol", PROTOCOL_VERSION)
        .body(data.into())
}
