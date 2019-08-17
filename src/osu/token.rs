use super::channel::Channel;
use super::matches::Match;
use super::GameMode;
use super::List;
use std::sync::{Arc, Mutex, RwLock, Weak};
use uuid::Uuid;

type DbConn = postgres::Connection;

// pub struct Panel {}

// impl Panel {
//     fn new() -> Self {
//         Panel{}
//     }
// }

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

//TODO: user stats
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
    fn new() -> Self {
        Stats {
            action: Action::Idle,
            action_text: "".to_string(),
            action_md5: "".to_string(),
            action_mods: 0,
            game_mode: GameMode::STANDARD,
            beatmap_id: 0,
            ranked_score: 0,
            accuracy: 1.0,
            playcount: 0,
            total_score: 0,
            rank: 1,
            pp: 0,
        }
    }

    // fn fetch(id: i32, mode: GameMode, db: &DbConn) -> Self {
    //     Self::new()
    // }

    //fn refresh(&mut self, id: i32, mode: GameMode, db: &DbConn) {}
}

#[derive(Debug)]
pub struct Token {
    data: Mutex<Vec<u8>>,
    token: String,
    id: u32,
    username: String,
    rank: u32,
    joined_channels: RwLock<Vec<Weak<Channel>>>,
    stats: RwLock<Stats>,
    multi: RwLock<Option<Weak<Match>>>, //location: [f32; 2]
}

impl Token {
    pub fn clear_queue(&self) -> Vec<u8> {
        let mut lock = self.data.lock().unwrap();
        let mut buf = Vec::with_capacity(lock.len());
        buf.append(&mut lock);
        trace!(
            "cleared queue - {:?} ({:?}, {:?})",
            self.token,
            self.id,
            self.username
        );
        buf
    }

    pub fn enqueue(&self, buf: &[u8]) {
        trace!(
            "enqueue data {:x?} - {:?} ({:?}, {:?})",
            buf,
            self.token,
            self.id,
            self.username
        );
        (*self.data.lock().unwrap()).extend_from_slice(buf);
    }

    pub fn token(&self) -> String {
        self.token.clone()
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn location(&self) -> &[f32] {
        &[0.0, 0.0]
    }

    pub fn rank(&self) -> u8 {
        38
    }

    pub fn leave_channel(&self, channel: &Arc<Channel>) {
        let mut channels = self.joined_channels.write().unwrap();
        match channels
            .iter()
            .position(|ch| Arc::ptr_eq(channel, &ch.upgrade().unwrap()))
        {
            Some(pos) => {
                channels.remove(pos);
                trace!("{:?} left {:?}", self.token, channel.name());
            }
            None => warn!(
                "{:?} tried leaving {:?} not being in it",
                self.token,
                channel.name()
            ),
        }
    }

    pub fn join_channel(&self, channel: Weak<Channel>) {
        self.joined_channels.write().unwrap().push(channel);
        trace!("{:?} joined new channel", self.token);
    }

    pub fn joined_channels(&self) -> std::sync::RwLockReadGuard<Vec<Weak<Channel>>> {
        self.joined_channels.read().unwrap()
    }

    pub fn refresh_stats(&self, _mode: GameMode, _db: &DbConn) {
        //TODO
        let mut _stats = self.stats.write().unwrap();
        //stats.refresh(self.id as _, mode, db);
    }

    pub fn read_stats(&self) -> std::sync::RwLockReadGuard<Stats> {
        self.stats.read().unwrap()
    }

    pub fn joim_match(&self, multi: Weak<Match>) {
        let mut m = self.multi.write().unwrap();
        *m = Some(multi);
    }
}

impl std::cmp::PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::cmp::PartialEq<u32> for Token {
    fn eq(&self, other: &u32) -> bool {
        self.id == *other
    }
}

impl std::cmp::PartialEq<i32> for Token {
    fn eq(&self, other: &i32) -> bool {
        self.id == *other as u32
    }
}

impl std::cmp::PartialEq<Arc<Token>> for Token {
    fn eq(&self, other: &Arc<Token>) -> bool {
        self.id == (*other).id()
    }
}

impl List<Token> {
    pub fn add_token(&self, id: u32, name: String) -> Arc<Token> {
        let token = Uuid::new_v4().to_string();
        let token = Token {
            id,
            username: name,
            rank: 38,
            token,
            data: Mutex::new(Vec::new()),
            joined_channels: RwLock::new(Vec::new()),
            stats: RwLock::new(Stats::new()),
            multi: RwLock::new(None),
        };
        let token = Arc::new(token);
        self.insert(token.token(), token.clone());
        trace!("new token inserted {:?}", token);
        token
    }

    pub fn has_id(&self, id: u32) -> bool {
        for token in self.list.read().unwrap().values().map(|t| t.clone()) {
            if *token == id {
                return true;
            }
        }
        false
    }

    pub fn enqueue_all(&self, data: &[u8]) {
        self.list
            .read()
            .unwrap()
            .values()
            .cloned()
            .for_each(|token| token.enqueue(data));
    }

    pub fn get_username(&self, username: &str) -> Option<Arc<Token>> {
        self.find(|&token| token.username == username)
    }
}
