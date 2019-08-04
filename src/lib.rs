#![feature(vec_remove_item)]

extern crate r2d2;

pub mod http;
pub mod osu;
mod events;
use osu::{List, packets};
use osu::token::Token;
use osu::channel::Channel;
use r2d2_postgres::PostgresConnectionManager as PgConnManager;
use r2d2_postgres::TlsMode;
use std::sync::Arc;

const EASTEREGG: &'static [u8] = b"
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
    pub db_pool: r2d2::Pool<PgConnManager>
    //pub req_count: AtomicUsize
}

impl Glob {
    pub fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let db_manager = PgConnManager::new(db_url, TlsMode::None).unwrap();
        let db_pool = r2d2::Pool::builder().build(db_manager).unwrap();
        Glob {
            token_list: List::new(), 
            channel_list: List::new(), db_pool
            //req_count: AtomicUsize::new(0)
        }
    }
}

use http::{request::Method, Request, Response};

fn login(req: &Request, glob: &Glob) -> (String, Vec<u8>) {
    let (username, password) = {
        let login_data: Vec<&str> = req.body_string().split('\n').collect();
        (login_data[0].trim(), login_data[1])
    };
    if username == "wojexe" {
        return ("0".to_string(), [osu::packets::server::login_failed(), osu::packets::server::notification("wojexe to ciota")].concat());
    }

    let conn = glob.db_pool.get().unwrap();
    let result = conn.query("SELECT id FROM users WHERE nick = $1 AND password = $2", &[&username, &password]).unwrap();
    if result.len() == 0 {
        return ("0".to_string(), osu::packets::server::login_failed());
    } else if result.len() > 1 {
        eprintln!("Found more than one result for this combination: {}:{}", username, password);
    };

    let id: i32 = result.get(0).get(0);
    println!("id: {}", id);
    let token = glob.token_list.add_token(id as u32, username.to_string());
    println!("{:?}", token);

    use packets::server as p;
    let data = [
        p::silence_end(0),
        p::protocol_ver(),
        p::user_id(token.id()),
        p::user_rank(0),

        p::friend_list(vec![]),

        p::user_panel(&token),
        p::user_stats(&token),
        
        //p::menu_icon("https://i.imgur.com/DmwAGYO.png"),

        p::online_users(glob.token_list.entries().into_iter().map(|token| token.id() as i32).collect()),
        //glob.token_list.entries().into_iter().flat_map(|token| p::user_panel(&token)).collect(),

        p::channel_info_end(),
        glob.channel_list.entries().into_iter().flat_map(|channel| p::channel_info(&channel)).collect(),
    ].concat();
    println!("{:x?}", data);

    (token.token(), data)
}

fn handle_event(req: &Request, token: &str, glob: &Glob) -> (String, Vec<u8>) {
    let user =
        if let Some(token) = glob.token_list.get(token) { token }
        else {
            return ("0".to_string(), osu::packets::server::login_failed());
        };

    //println!("Handling request from user {}", user.username());
    
    use packets::client::ID;
    let request_data = req.body();
    let mut pos = 0;
    println!("Request data: {}\n{:x?}", request_data.len(), request_data);
    while pos < request_data.len() {
        let (id, len, data) = packets::client::parse_packet(&request_data[pos..]);
        match id {
            ID::UNKNOWN => eprintln!("Found an unknown packet!\n{:x?}\n", data),
            ID::SEND_PUBLIC_MESSAGE => events::send_public_message(data, &user, glob),
            ID::LOGOUT => events::logout(token, glob),
            ID::PONG => (),
            ID::CHANNEL_JOIN => events::channel_join(data, user.clone(), glob),
            ID::USER_STATS_REQUEST => events::user_stats_request(data, &user, glob),
            ID::USER_PRESENCE_REQUEST => events::user_panel_request(data, &user, glob),
            _ => eprintln!("Unhandled packet! {:?}\n{:x?\n}", id, data)
        }
        pos += 7 + len as usize;
    }

    let data = user.clear_queue();
    if data.len() > 0 { println!("{:x?}", data) }
    (user.token(), data)
}

fn osu_packet(req: &Request, glob: &Glob) -> Response {
    let (token, data) = match req.get_header("osu-token") {
        None => login(req, glob),
        Some(&token) => handle_event(req, token, glob)
    };
    
    println!("=======RAW\nlength: {}\n{}\n{:x?}\n========", data.len(), String::from_utf8_lossy(&data), data);
    
    let mut res = Response::from(data.as_ref());
    res.put_headers(&[
        ("cho-token", &token),
        ("cho-protocol", "19"),
        ("Keep-Alive", "timeout=5, max=100"),
        ("Connection", "keep-alive"),
        ("Content-Type", "text/html; charset=UTF-8")
    ]);
    //res.log_it();
    res
}

fn main_handler(req: &Request, glob: &Glob) -> Response {
    match req.path() {
        "/" => match req.method() {
            Method::GET => Response::from(EASTEREGG),
            Method::POST => osu_packet(req, glob),
            Method::OTHER(_o) => Response::empty_nf()
        },
        _path => Response::empty_nf()
    }
}

