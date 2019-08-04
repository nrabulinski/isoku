use std::sync::{RwLock, Arc};
use super::token::Token;
use super::List;
use super::packets::server as packets;

#[derive(Debug)]
pub struct Channel {
    name: String,
    desc: String,
    users: RwLock<Vec<Arc<Token>>>
}

impl Channel {
    pub fn new(name: String, desc: String) -> Self {
        Channel {
            name, desc,
            users: RwLock::new(Vec::new())
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn desc(&self) -> &str {
        &self.desc
    }

    pub fn users_len(&self) -> u16 {
        self.users.read().unwrap().len() as u16
    }

    pub fn add_client(&self, token: Arc<Token>) {
        if self.users.read().unwrap().contains(&token) {
            return
        };

        token.enqueue(&packets::channel_join_success(self.name()));
        self.users.write().unwrap().push(token);
    }
    
    pub fn remove_client(&self, token: &Arc<Token>) {
        self.users.write().unwrap().remove_item(token);
    }

    pub fn client_name(&self) -> &str {
        if self.name.starts_with("#spect_") {
            "#spectator"
        } else if self.name.starts_with("#multi_") {
            "#multiplayer"
        } else {
            &self.name
        }
    }

    pub fn users(&self) -> std::sync::RwLockReadGuard<Vec<Arc<Token>>> {
        self.users.read().unwrap()
    }
}

impl List<Channel> {
    pub fn add_channel(&self, name: String, desc: String) {
        let channel = Arc::new(Channel::new(name.clone(), desc));
        self.list.write().unwrap().insert(name, channel);
    }
}