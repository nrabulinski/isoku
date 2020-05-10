use super::Token;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct Channel {
    pub name: String,
    pub desc: String,
    pub users: RwLock<Vec<Arc<Token>>>,
    pub public: bool,
}

impl Channel {
    pub fn new(
        list: &mut HashMap<String, Arc<Channel>>,
        name: &str,
        desc: &str,
        public: bool,
    ) -> Arc<Channel> {
        let res = Channel {
            name: name.to_string(),
            desc: desc.to_string(),
            users: RwLock::default(),
            public,
        };
        let res = Arc::new(res);
        list.insert(name.to_string(), res.clone());
        res
    }

    pub fn name(&self) -> &str {
        if self.name.starts_with("#multi") {
            "#multiplayer"
        } else if self.name.starts_with("#spect") {
            "#spectator"
        } else {
            &self.name
        }
    }

    pub async fn has_user(&self, token: &Arc<Token>) -> bool {
        for c in self.users.read().await.iter() {
            if Arc::ptr_eq(c, token) {
                return true;
            }
        }
        false
    }

    pub async fn user_join(&self, token: Arc<Token>) -> bool {
        if self.has_user(&token).await {
            return false;
        }

        self.users.write().await.push(token);
        true
    }

    pub async fn user_part(&self, token: &Arc<Token>) -> bool {
        let mut users = self.users.write().await;
        if let Some(pos) = users.iter().position(|t| Arc::ptr_eq(t, token)) {
            users.remove(pos);
            true
        } else {
            false
        }
    }
}
