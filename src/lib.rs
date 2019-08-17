//TODO: Remove all useless comments
//TODO: Proper logging system

#[macro_use]
extern crate log;
extern crate r2d2;
pub mod bytes;
mod events;
pub mod http;
pub mod osu;
use bytes::Cursor;
use osu::channel::Channel;
use osu::matches::Match;
use osu::token::Token;
use osu::{packets, GameMode, List};
use r2d2_postgres::PostgresConnectionManager as PgConnManager;
use r2d2_postgres::TlsMode;
use std::sync::{RwLock, Weak};

const EASTEREGG: &[u8] = b"
<html>
<head>
<title>Uncho</title>
</head>
<body>
<pre>
                    __        
  __  ______  _____/ /_  ____ 
 / / / / __ \\/ ___/ __ \\/ __ \\
/ /_/ / / / / /__/ / / / /_/ /
\\__,_/_/ /_/\\___/_/ /_/\\____/
world's first osu private server written in Rust
</pre>
</body>
</html>";

pub struct Glob {
    pub token_list: List<Token>,
    pub channel_list: List<Channel>,
    pub db_pool: r2d2::Pool<PgConnManager>,
    pub match_list: List<Match>,
    pub menu_icon: RwLock<Option<String>>,
    pub lobby: RwLock<Vec<Weak<Token>>>,
}

impl Glob {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let db_manager = PgConnManager::new(db_url, TlsMode::None).unwrap();
        let db_pool = r2d2::Pool::builder().build(db_manager).unwrap();
        Glob {
            token_list: List::new(),
            channel_list: List::new(),
            match_list: List::new(),
            db_pool,
            menu_icon: RwLock::new(None),
            lobby: RwLock::new(Vec::new()),
        }
    }
}

use http::{request::Method, Request, Response};

fn login(req: &Request, glob: &Glob) -> (String, Vec<u8>) {
    let failure = ("0".to_string(), osu::packets::server::login_failed());
    let (username, password) = {
        let login_data: Vec<&str> = req.body_string().split('\n').collect();
        if login_data.len() < 2 {
            warn!(
                "didn't recieve enough data from {} {:?}",
                req.ip(),
                login_data
            );
            return failure;
        }
        (login_data[0].trim(), login_data[1])
    };
    trace!(
        "login request from {} for {:?}",
        req.get_header("x-real-ip").unwrap_or(&req.ip().as_str()),
        username
    );
    if username == "wojexe" {
        return (
            "0".to_string(),
            [
                osu::packets::server::login_failed(),
                osu::packets::server::notification("wojexe to ciota"),
            ]
            .concat(),
        );
    }

    let conn = glob.db_pool.get().unwrap();
    let result = conn
        .query(
            "SELECT id FROM users WHERE nick = $1 AND password = $2",
            &[&username, &password],
        )
        .unwrap();
    if result.is_empty() {
        return failure;
    } else if result.len() > 1 {
        error!(
            "Found more than one result for this combination: {}:{}",
            username, password
        );
    };

    let id: i32 = result.get(0).get(0);
    let token = glob.token_list.add_token(id as u32, username.to_string());
    token.refresh_stats(GameMode::STANDARD, &conn);

    let online: Vec<i32> = glob
        .token_list
        .entries()
        .into_iter()
        .map(|token| token.id() as i32)
        .collect();

    use packets::server as p;
    let mut data = [
        p::silence_end(0),
        p::protocol_ver(),
        p::user_id(token.id()),
        p::user_rank(0),
        p::friend_list(&[]),
        p::user_panel(&token),
        p::user_stats(&token),
        p::online_users(&online),
        //below some threshold we can just append all users' panels I guess
        glob.token_list
            .entries()
            .into_iter()
            .flat_map(|token| p::user_panel(&token))
            .collect(),
        p::channel_info_end(),
        glob.channel_list
            .entries()
            .into_iter()
            .filter(|channel| channel.public())
            .flat_map(|channel| p::channel_info(&channel))
            .collect(),
    ]
    .concat();

    let menu_icon = glob.menu_icon.read().unwrap();
    if menu_icon.is_some() {
        data.extend(p::menu_icon(menu_icon.as_ref().unwrap()));
    }

    glob.token_list.enqueue_all(&p::user_panel(&token));

    (token.token(), data)
}

fn handle_event(req: &Request, token: &str, glob: &Glob) -> (String, Vec<u8>) {
    let user = if let Some(token) = glob.token_list.get(token) {
        token
    } else {
        return ("0".to_string(), osu::packets::server::login_failed());
    };

    let request_data = req.body();
    let mut c = Cursor::new(request_data);
    trace!(
        "handling request from {:?} @ {}: {:x?}",
        token,
        req.ip(),
        request_data
    );
    use packets::client::ID;
    while c.remaining() >= 7 {
        let (id, mut data) = packets::client::parse_packet(&mut c);
        trace!("({:?}) parsed packet {:?} {:x?}", token, id, data.data());
        match id {
            ID::UNKNOWN => warn!("unknown packet {:x?}", data.data()),
            ID::CHANGE_ACTION => events::change_action(&mut data, &user, glob),
            ID::SEND_PUBLIC_MESSAGE => events::send_public_message(&mut data, &user, glob),
            ID::LOGOUT => events::logout(token, glob),
            ID::PONG => (),
            ID::SEND_PRIVATE_MESSAGE => events::send_private_message(&mut data, &user, glob),
            ID::PART_LOBBY => events::part_lobby(&user, glob),
            ID::JOIN_LOBBY => events::join_lobby(&user, glob),
            ID::CREATE_MATCH => events::create_match(&mut data, &user, glob),
            ID::CHANNEL_JOIN => events::channel_join(&mut data, &user, glob),
            ID::CHANNEL_PART => events::channel_part(&mut data, &user, glob),
            ID::USER_STATS_REQUEST => events::user_stats_request(&mut data, &user, glob),
            ID::USER_PANEL_REQUEST => events::user_panel_request(&mut data, &user, glob),
            _ => warn!("unhandled packet {:?}", id),
        }
    }

    if c.remaining() > 0 {
        warn!(
            "{:?} sent more data than could be parsed {:x?}",
            token,
            c.data()
        );
    }

    let data = user.clear_queue();
    (user.token(), data)
}

fn osu_packet(req: &Request, glob: &Glob) -> Response {
    let (token, data) = match req.get_header("osu-token") {
        None => login(req, glob),
        Some(&token) => handle_event(req, token, glob),
    };

    let mut res = Response::from(data.as_ref());
    res.put_headers(&[
        ("cho-token", &token),
        ("cho-protocol", "19"),
        ("Keep-Alive", "timeout=5, max=100"),
        ("Connection", "keep-alive"),
        ("Content-Type", "text/html; charset=UTF-8"),
    ]);
    res
}

fn main_handler(req: &Request, glob: &Glob) -> Response {
    match req.path() {
        "/" => match req.method() {
            Method::GET => Response::from(EASTEREGG),
            Method::POST => osu_packet(req, glob),
            Method::OTHER(_o) => Response::empty_nf(),
        },
        _path => Response::empty_nf(),
    }
}
