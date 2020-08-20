#![allow(dead_code)]
use crate::{packets::OsuEncode, Token};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering},
        Arc,
    },
};
use tokio::sync::RwLock;

enum_try_from!(
    #[repr(u8)]
    #[derive(Debug, Clone, Copy)]
    pub enum SlotStatus {
        Free = 1,
        Locked = 2,
        NotReady = 4,
        Ready = 8,
        NoMap = 16,
        Playing = 32,
        Occupied = 124,
        PlayingQuit = 128,
    }
);

impl SlotStatus {
    pub fn is_occupied(self) -> bool { self as u8 & SlotStatus::Occupied as u8 > 0 }
}

enum_try_from!(
    #[repr(u8)]
    #[derive(Debug, Clone, Copy)]
    pub enum Team {
        NoTeam = 0,
        Blue = 1,
        Red = 2,
    }
);

enum_try_from!(
    #[repr(u8)]
    #[derive(Debug, Clone, Copy)]
    pub enum ScoringType {
        Score,
        Accuracy,
        Combo,
        ScoreV2,
    }
);

enum_try_from!(
    #[repr(u8)]
    #[derive(Debug, Clone, Copy)]
    pub enum TeamType {
        HeadToHead,
        TagCoop,
        TeamVs,
        TagTeamVs,
    }
);

#[derive(Debug)]
pub struct Slot {
    pub status: AtomicU8, //SlotStatus,
    pub team: AtomicU8,   //Team,
    pub token: RwLock<Option<Arc<dyn Token>>>,
    pub skip: AtomicBool,
    pub mods: AtomicU32,
}

impl Default for Slot {
    fn default() -> Self {
        Slot {
            status: AtomicU8::new(SlotStatus::Free as u8),
            team: AtomicU8::new(Team::NoTeam as u8),
            token: RwLock::default(),
            skip: AtomicBool::default(),
            mods: AtomicU32::new(0),
        }
    }
}

#[derive(Debug)]
pub struct Match {
    pub id: u16,
    pub name: RwLock<String>,
    pub password: RwLock<Option<String>>,
    pub slots: [Slot; 16],
    pub in_progress: AtomicBool,
    pub mods: AtomicU32,
    pub beatmap_id: AtomicU32,
    pub beatmap_name: RwLock<String>,
    pub beatmap_md5: RwLock<String>,
    pub host_id: RwLock<i32>,
    pub game_mode: AtomicU8,
    pub scoring_type: AtomicU8,
    pub team_type: AtomicU8,
    pub freemod: AtomicBool,
}

impl Match {
    pub async fn new(
        list: &mut HashMap<u16, Arc<Match>>,
        name: &str,
        password: &str,
        beatmap_name: &str,
        beatmap_id: u32,
        beatmap_md5: &str,
        owner: &Arc<dyn Token>,
    ) -> Arc<Self> {
        let slots = [
            Slot {
                status: AtomicU8::new(SlotStatus::NotReady as u8),
                token: RwLock::new(Some(owner.clone())),
                ..Slot::default()
            },
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
            Slot::default(),
        ];
        let res = Match {
            id: list.keys().count() as u16,
            name: RwLock::new(name.to_string()),
            password: RwLock::new(if password.is_empty() {
                None
            } else {
                Some(password.to_string())
            }),
            slots,
            in_progress: AtomicBool::new(false),
            mods: AtomicU32::new(0),
            beatmap_id: AtomicU32::new(beatmap_id),
            beatmap_name: RwLock::new(beatmap_name.to_string()),
            beatmap_md5: RwLock::new(beatmap_md5.to_string()),
            host_id: RwLock::new(owner.id()),
            game_mode: AtomicU8::new(0),
            scoring_type: AtomicU8::new(ScoringType::Score as u8),
            team_type: AtomicU8::new(TeamType::HeadToHead as u8),
            freemod: AtomicBool::new(false),
        };
        let res = Arc::new(res);
        list.insert(res.id, res.clone());
        res
    }

    pub fn slot_statuses(&self) -> Vec<u8> {
        self.slots
            .iter()
            .map(|slot| slot.status.load(Ordering::SeqCst))
            .collect()
    }

    pub fn slot_teams(&self) -> Vec<u8> {
        self.slots
            .iter()
            .map(|slot| slot.team.load(Ordering::SeqCst))
            .collect()
    }

    pub async fn slot_ids(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(16 * 4);
        for slot in self.slots.iter() {
            let token = slot.token.read().await.clone();
            if let Some(t) = token {
                t.id().encode(&mut res);
            }
        }
        res
    }

    pub fn slot_mods(&self) -> Vec<u8> {
        if !self.freemod.load(Ordering::SeqCst) {
            return Vec::new();
        }
        let mut res = Vec::with_capacity(16 * 4);
        self.slots
            .iter()
            .for_each(|slot| slot.mods.load(Ordering::SeqCst).encode(&mut res));
        res
    }
}
