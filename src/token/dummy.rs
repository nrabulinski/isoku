use super::Token;
use crate::Channel;
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
use tokio::sync::{RwLock, RwLockReadGuard};
use uuid::Uuid;

pub struct DummyToken {
    id: i32,
    token: String,
    username: String,
    channels: RwLock<Vec<Weak<Channel>>>,
}

impl DummyToken {
    pub fn new(
        list: &mut HashMap<String, Arc<dyn Token>>,
        id: i32,
        username: String,
    ) -> Arc<dyn Token> {
        let token = Uuid::new_v4().to_hyphenated().to_string();
        let res = DummyToken {
            id,
            token: token.clone(),
            username,
            channels: RwLock::default(),
        };
        let res = Arc::new(res);
        list.insert(token, res.clone());
        res
    }
}

#[async_trait]
impl Token for DummyToken {
    fn id(&self) -> i32 { self.id }

    fn token(&self) -> &str { &self.token }

    fn username(&self) -> &str { &self.username }

    async fn enqueue(&self, buf: &[u8]) { () }

    async fn enqueue_vec(&self, buf: Vec<u8>) { () }

    async fn join_channel(&self, ch: Weak<Channel>) { self.channels.write().await.push(ch); }

    async fn channels(&self) -> RwLockReadGuard<'_, Vec<Weak<Channel>>> {
        self.channels.read().await
    }
}
