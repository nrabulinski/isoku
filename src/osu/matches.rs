use super::token::Token;
use super::GameMode;
use super::List;
use std::sync::{Arc, RwLock};

type DbConn = postgres::Connection;

#[derive(Debug, Copy, Clone)]
enum Team {
    None,
    Blue,
    Red,
}
#[derive(Debug, Copy, Clone, PartialEq)]
enum Status {
    Free = 1,
    Locked = 1 << 1,
    NotReady = 1 << 2,
    Ready = 1 << 3,
    NoMap = 1 << 4,
    Playing = 1 << 5,
    Occupied = 1 << 6,
    PlayingQuit = 1 << 7,
}
#[derive(Debug, Copy, Clone)]
enum ScoreType {
    Score,
    Accuracy,
    Combo,
    ScoreV2,
}
#[derive(Debug, Copy, Clone)]
enum TeamType {
    HeadToHead,
    TagCOOP,
    TeamVS,
    TagTeamVS,
}

#[derive(Debug)]
pub struct Slot {
    user: Option<Arc<Token>>,
    completed: bool,
    passed: bool,
    skip: bool,
    mods: u32,
    score: u64,
    loaded: bool,
    team: Team,
    status: Status,
}

impl Slot {
    fn new(free: bool) -> Slot {
        Slot {
            user: None,
            completed: false,
            passed: false,
            skip: false,
            mods: 0,
            score: 0,
            loaded: false,
            team: Team::None,
            status: if free { Status::Free } else { Status::Locked },
        }
    }
}

#[derive(Debug)]
pub struct Match {
    id: i32,
    slots: RwLock<Vec<Slot>>,
    in_progress: bool,
    mods: u32,
    name: String,
    password: String,
    beatmap_id: i32,
    beatmap_name: String,
    beatmap_md5: String,
    host_user_id: u32,
    game_mode: GameMode,
    score_type: ScoreType,
    team_type: TeamType,
    freemod: bool,
}

impl Match {
    pub fn new(
        id: i32,
        name: String,
        password: String,
        beatmap_id: i32,
        beatmap_name: String,
        beatmap_md5: String,
        game_mode: u8,
        host_user_id: u32,
    ) -> Match {
        let mut slots = Vec::with_capacity(16);
        for _ in 0..16 {
            slots.push(Slot::new(true))
        }
        let slots = RwLock::new(slots);
        Match {
            id,
            name,
            password,
            beatmap_id,
            beatmap_name,
            beatmap_md5,
            game_mode: GameMode::n(game_mode).unwrap_or(GameMode::STANDARD),
            slots,
            in_progress: false,
            mods: 0,
            score_type: ScoreType::Score,
            team_type: TeamType::HeadToHead,
            freemod: false,
            host_user_id,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn in_progress(&self) -> u8 {
        if self.in_progress {
            1
        } else {
            0
        }
    }

    pub fn mods(&self) -> u32 {
        self.mods
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn password(&self, censored: bool) -> String {
        if censored && !self.password.is_empty() {
            "redacted".to_string()
        } else {
            self.password.clone()
        }
    }

    pub fn beatmap_name(&self) -> String {
        self.beatmap_name.clone()
    }

    pub fn beatmap_id(&self) -> i32 {
        self.beatmap_id
    }

    pub fn beatmap_md5(&self) -> String {
        self.beatmap_md5.clone()
    }

    pub fn slots(&self) -> Vec<u8> {
        self.slots
            .read()
            .unwrap()
            .iter()
            .map(|slot| slot.status as u8)
            .collect()
    }

    pub fn teams(&self) -> Vec<u8> {
        self.slots
            .read()
            .unwrap()
            .iter()
            .map(|slot| slot.team as u8)
            .collect()
    }

    pub fn users(&self) -> Vec<u8> {
        use crate::bytes::AsBuf;
        let slots: Vec<u32> = self
            .slots
            .read()
            .unwrap()
            .iter()
            .filter(|slot| slot.user.is_some())
            .map(|slot| slot.user.as_ref().unwrap().id())
            .collect();
        let mut buf: Vec<u8> = Vec::with_capacity(4 * slots.len());
        for slot in slots {
            slot.encode(&mut buf);
        }
        buf
    }

    pub fn host(&self) -> u32 {
        self.host_user_id
    }

    pub fn game_mode(&self) -> u8 {
        self.game_mode as u8
    }

    pub fn score_type(&self) -> u8 {
        self.score_type as u8
    }

    pub fn team_type(&self) -> u8 {
        self.team_type as u8
    }

    pub fn freemod(&self) -> u8 {
        if self.freemod {
            1
        } else {
            0
        }
    }

    pub fn slot_mods(&self) -> Vec<u8> {
        if !self.freemod {
            return Vec::new();
        }
        use crate::bytes::AsBuf;
        let slots: Vec<u32> = self
            .slots
            .read()
            .unwrap()
            .iter()
            .map(|slot| slot.mods)
            .collect();
        let mut buf: Vec<u8> = Vec::with_capacity(4 * slots.len());
        for slot in slots {
            slot.encode(&mut buf);
        }
        buf
    }

    pub fn user_join(&self, token: Arc<Token>) -> bool {
        match self
            .slots
            .write()
            .unwrap()
            .iter_mut()
            .find(|slot| slot.status == Status::Free)
        {
            Some(slot) => {
                slot.user = Some(token);
                slot.status = Status::NotReady;
                true
            }
            None => false,
        }
    }
}

impl List<Match> {
    pub fn create_match(
        &self,
        name: String,
        password: String,
        beatmap_id: i32,
        beatmap_name: String,
        beatmap_md5: String,
        game_mode: u8,
        host_user_id: u32,
        db: &DbConn,
    ) -> Arc<Match> {
        let result = db
            .query(
                "INSERT INTO matches (name) VALUES ($1) RETURNING id",
                &[&name],
            )
            .unwrap();
        let id: i32 = result.get(0).get(0);
        let multi = Match::new(
            id,
            name,
            password,
            beatmap_id,
            beatmap_name,
            beatmap_md5,
            game_mode,
            host_user_id,
        );
        let multi = Arc::new(multi);
        self.insert(id.to_string(), multi.clone());
        trace!("Created new match {:?}", multi);
        multi
    }
}
