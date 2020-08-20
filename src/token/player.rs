use super::Token;
use crate::{Channel, Glob, Match};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
    time::Duration,
};
use tokio::{
    sync::{mpsc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::timeout,
};
use uncho_common::TryFrom;
use uuid::Uuid;

#[repr(u8)]
#[allow(dead_code)]
#[derive(TryFrom, Debug, Copy, Clone)]
pub enum GameMode {
    Standard = 0,
    Taiko = 1,
    Ctb = 2,
    Mania = 3,
}

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

impl Stats {
    pub const fn new() -> Self {
        Stats {
            action: Action::Idle,
            action_text: String::new(),
            action_md5: String::new(),
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

impl Default for Stats {
    fn default() -> Self { Stats::new() }
}

#[derive(Debug)]
pub struct PlayerToken {
    pub id: i32,
    pub token: String,
    pub username: String,
    pub queue: Mutex<Vec<u8>>,
    pub stats: RwLock<Stats>,
    pub channels: RwLock<Vec<Weak<Channel>>>,
    pub multi: Mutex<Option<Weak<Match>>>,
    pub sender: Option<Mutex<mpsc::Sender<&'static str>>>,
}

impl PlayerToken {
    pub fn new(
        list: &mut HashMap<String, Arc<dyn Token>>,
        id: i32,
        username: String,
    ) -> Arc<dyn Token> {
        let token = Uuid::new_v4().to_hyphenated().to_string();
        let res = PlayerToken {
            id,
            token: token.clone(),
            queue: Mutex::new(Vec::new()),
            username,
            stats: RwLock::default(),
            channels: RwLock::default(),
            multi: Mutex::default(),
            sender: None,
        };
        let res = Arc::new(res);
        list.insert(token, res.clone());
        res
    }

    pub async fn new_with_timeout(
        glob: Arc<Glob>,
        id: i32,
        username: String,
        duration: Duration,
    ) -> Arc<dyn Token> {
        let token = Uuid::new_v4().to_hyphenated().to_string();
        let (sender, mut recv) = mpsc::channel(1);
        let token2 = token.clone();
        let res = PlayerToken {
            id,
            token: token.clone(),
            queue: Mutex::new(Vec::new()),
            username,
            stats: RwLock::default(),
            channels: RwLock::default(),
            multi: Mutex::default(),
            sender: Some(Mutex::new(sender)),
        };
        let res = Arc::new(res);
        glob.token_list.write().await.insert(token, res.clone());
        tokio::spawn(async move {
            let duration = duration;
            let glob = glob;
            let token = token2;
            loop {
                match timeout(duration.clone(), recv.recv()).await {
                    Ok(Some(val)) if val == "ping" => continue,
                    _ => {
                        crate::events::logout::handle(&token, &glob).await.ok();
                        break;
                    }
                }
            }
        });
        res
    }
}

#[async_trait]
impl Token for PlayerToken {
    fn id(&self) -> i32 { self.id }

    async fn stats(&self) -> RwLockReadGuard<'_, Stats> { self.stats.read().await }

    async fn stats_mut(&self) -> Option<RwLockWriteGuard<'_, Stats>> {
        Some(self.stats.write().await)
    }

    fn token(&self) -> &str { &self.token }

    fn username(&self) -> &str { &self.username }

    async fn join_channel(&self, ch: Weak<Channel>) { self.channels.write().await.push(ch) }

    async fn enqueue(&self, buf: &[u8]) { self.queue.lock().await.extend_from_slice(buf) }

    async fn enqueue_vec(&self, mut buf: Vec<u8>) { self.queue.lock().await.append(&mut buf) }

    async fn channels(&self) -> RwLockReadGuard<'_, Vec<Weak<Channel>>> {
        self.channels.read().await
    }

    fn as_player(&self) -> Option<&PlayerToken> { Some(self) }

    async fn clear_queue(&self) -> Vec<u8> {
        let mut m = self.queue.lock().await;
        let mut res = Vec::with_capacity(m.len());
        res.append(&mut m);
        res
    }
}
