pub mod dummy;
pub mod player;
use crate::Channel;
use async_trait::async_trait;
use core::{
    fmt,
    fmt::{Debug, Formatter},
};
use lazy_static::lazy_static;
use player::{Action, Stats};
use std::sync::Weak;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
// use std::future::Future;

lazy_static! {
    static ref DEFAULT_STATS: RwLock<Stats> = {
        let mut stats = Stats::new();
        stats.action = Action::Testing;
        stats.action_text = "I'm a bot!".to_string();
        RwLock::new(stats)
    };
}

#[async_trait]
pub trait Token: Send + Sync {
    fn id(&self) -> i32;
    async fn stats(&self) -> RwLockReadGuard<'_, Stats> { DEFAULT_STATS.read().await }
    async fn stats_mut(&self) -> Option<RwLockWriteGuard<'_, Stats>> { None }
    fn token(&self) -> &str;
    fn username(&self) -> &str;
    async fn enqueue(&self, buf: &[u8]);
    // type Enqueue<'a>: Future<Output=()> + 'a;
    // fn enqueue<'s, 'b, 'a>(&'s self, buf: &'b [u8]) -> Self::Enqueue<'a>
    //     where 's: 'a, 'b: 'a;
    async fn enqueue_vec(&self, buf: Vec<u8>);
    // type EnqueueVec<'a>: Future<Output=()> + 'a;
    // fn enqueue_vec(&self, buf: Vec<u8>) -> Self::EnqueueVec<'_>;
    async fn join_channel(&self, ch: Weak<Channel>);
    async fn channels(&self) -> RwLockReadGuard<'_, Vec<Weak<Channel>>>;
    fn as_player(&self) -> Option<&player::PlayerToken> { None }
    async fn clear_queue(&self) -> Vec<u8> { Vec::new() }
}

impl Debug for dyn Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("OsuToken");
        s.field("id", &self.id());
        s.field("token", &self.token());
        s.field("username", &self.username());
        s.finish()
    }
}
