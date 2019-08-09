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

    pub fn add_client(&self, token: Arc<Token>) -> bool {
        if self.has_client(&token) {
            return false;
        }

        self.users.write().unwrap().push(token);
        trace!("{:?} new client joined", self.name);
        true
    }
    
    pub fn remove_client(&self, token: &Arc<Token>) {
        let mut users = self.users.write().unwrap();
        if let Some(pos) = users.iter().position(|t| Arc::ptr_eq(t, token)) {
            users.remove(pos);
            trace!("removed {:?} from {:?}", token.token(), self.name);
        } else {
            warn!("tried to remove {:?} from {:?} before they joined it", token.token(), self.name);
        }
    }

    pub fn has_client(&self, token: &Arc<Token>) -> bool {
        for client in self.users.read().unwrap().iter() {
            if Arc::ptr_eq(client, token) {
                return true;
            }
        }
        false
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