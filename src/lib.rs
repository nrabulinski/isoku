#![feature(async_closure, const_generics)]
#![deny(bare_trait_objects)]
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
pub use token::{dummy::DummyToken, player::PlayerToken};
pub mod channel;
pub use channel::Channel;
pub mod bot;
pub mod events;
pub mod r#match;
pub use r#match::Match;

use token::Token;

const PROTOCOL_VERSION: u32 = 19;

#[derive(Debug)]
pub struct Glob {
    pub db_pool: PgPool,
    pub token_list: RwLock<HashMap<String, Arc<dyn Token>>>,
    pub channel_list: RwLock<HashMap<String, Arc<Channel>>>,
    pub match_list: RwLock<HashMap<u16, Arc<Match>>>,
    pub lobby: RwLock<Vec<Arc<dyn Token>>>,
    pub bot: Arc<dyn Token>,
}

impl Glob {
    pub async fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let db_pool = PgPool::new(&db_url).await.unwrap();
        let mut channel_list = HashMap::with_capacity(2);
        Channel::new(&mut channel_list, "#osu", "wojexe to ciota", true);
        Channel::new(&mut channel_list, "#lobby", "multi", true);
        let mut token_list = HashMap::with_capacity(2);
        let bot = DummyToken::new(&mut token_list, 3, "kenshichi".to_string());
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
    if glob.token_list.read().await.values().any(|t| t.id() == id) {
        return Err("Already logged in?");
    }
    let token = {
        // let mut list = glob.token_list.write().await;
        // Token::new(&mut list, id, username.to_string())
        PlayerToken::new_with_timeout(
            glob.clone(),
            id,
            username.to_string(),
            std::time::Duration::from_secs(60 * 5),
        )
        .await
    };
    let online: Vec<i32> = glob
        .token_list
        .read()
        .await
        .values()
        .map(|t| t.id())
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
        .flat_map(|t| p::user_panel(t.as_ref()))
        .collect();
    let user_stats = p::user_stats(token.as_ref()).await;
    let data = [
        p::silence_end(0),
        p::protocol_ver(PROTOCOL_VERSION),
        p::user_id(token.id()),
        p::user_rank(0),
        p::friend_list(&[]),
        p::user_panel(token.as_ref()),
        user_stats,
        p::online_users(&online),
        token_list,
        p::channel_info_end(),
        channels.into_iter().flatten().collect(),
    ]
    .concat();
    Ok((token.token().to_owned(), data))
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
        let packet: Result<(), String> = match id {
            Id::Unknown => {
                println!("UNKNOWN ID!");
                continue;
            }
            Id::SendPublicMessage => events::send_message::public(data, &token, &glob).await,
            Id::SendPrivateMessage => events::send_message::private(data, &token, &glob).await,
            Id::Pong => {
                if let Some(player) = token.as_player() {
                    if let Some(m) = player.sender.as_ref() {
                        let mut m = m.lock().await;
                        if let Err(_) = m.send("ping").await {
                            error!("Couldn't send ping");
                            events::logout::handle(token.token(), &glob)
                                .await
                                .map_err(|e| {
                                    error!(e);
                                })
                                .ok();
                        }
                    }
                }
                continue;
            }
            Id::UserStatsRequest => {
                events::stats_request::handle(data, token.as_ref(), &glob).await
            }
            Id::RequestStatusUpdate => events::status_update::handle(token.as_ref()).await,
            Id::CreateMatch => events::matches::create(data, &token, &glob).await,
            Id::MatchChangeSettings => {
                events::matches::change_settings(data, token.as_ref(), &glob).await
            }
            Id::JoinLobby => events::lobby_join::handle(&token, &glob).await,
            Id::PartLobby => events::lobby_part::handle(&token, &glob).await,
            Id::ChannelJoin => events::channel_join::handle(data, &token, &glob).await,
            Id::ChannelPart => events::channel_part::handle(data, &token, &glob).await,
            Id::ChangeAction => events::change_action::handle(data, token.as_ref(), &glob).await,
            Id::Logout => {
                events::logout::handle(token.token(), &glob).await?;
                break;
            }
            _ => Err(format!("Unhandled packet {:?}", id)),
        };
        if let Err(msg) = packet {
            res.append(&mut p::notification(&msg));
        }
    }
    res.append(&mut token.clear_queue().await);
    Ok((token.token().to_owned(), res))
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
