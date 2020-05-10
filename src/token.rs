use crate::{Channel, Match};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

enum_try_from!(
    #[repr(u8)]
    #[allow(dead_code)]
    #[derive(Debug, Copy, Clone)]
    pub enum GameMode {
        Standard,
        Taiko,
        Ctb,
        Mania,
    }
);

enum_try_from!(
    #[repr(u8)]
    #[allow(dead_code)]
    #[derive(Debug, Copy, Clone)]
    pub enum Action {
        Idle,
        Afk,
        Playing,
        Editing,
        Modding,
        Multiplayer,
        Watching,
        Unknown,
        Testing,
        Submitting,
        Paused,
        Lobby,
        Multiplaying,
        OsuDirect,
        None,
    }
);

#[derive(Debug)]
pub struct Stats {
    pub action: Action,
    pub action_text: String,
    pub action_md5: String,
    pub action_mods: u32,
    pub game_mode: GameMode,
    pub beatmap_id: u32,
    pub ranked_score: u64,
    pub accuracy: f32,
    pub playcount: u32,
    pub total_score: u64,
    pub rank: u32,
    pub pp: u16,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            action: Action::Idle,
            action_text: "".to_string(),
            action_md5: "".to_string(),
            action_mods: 0,
            game_mode: GameMode::Standard,
            beatmap_id: 0,
            ranked_score: 0,
            accuracy: 1.0,
            playcount: 0,
            total_score: 0,
            rank: 1,
            pp: 0,
        }
    }
}

#[derive(Debug)]
pub struct Token {
    pub id: i32,
    pub token: String,
    pub username: String,
    pub queue: Mutex<Vec<u8>>,
    pub stats: RwLock<Stats>,
    pub channels: Mutex<Vec<Weak<Channel>>>,
    pub multi: Mutex<Option<Weak<Match>>>,
}

impl Token {
    pub fn new(list: &mut HashMap<String, Arc<Token>>, id: i32, username: String) -> Arc<Token> {
        let token = Uuid::new_v4().to_hyphenated().to_string();
        let res = Token {
            id,
            token: token.clone(),
            queue: Mutex::new(Vec::new()),
            username,
            stats: RwLock::default(),
            channels: Mutex::default(),
            multi: Mutex::default(),
        };
        let res = Arc::new(res);
        list.insert(token, res.clone());
        res
    }

    pub async fn join_channel(&self, ch: Weak<Channel>) { self.channels.lock().await.push(ch) }
}
