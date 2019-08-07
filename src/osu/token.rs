use std::sync::{Arc, Mutex, RwLock, Weak};
use super::List;
use super::channel::Channel;
//use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct Token {
    data: Mutex<Vec<u8>>,
    token: String,
    id: u32,
    username: String,
    rank: u32,
    joined_channels: RwLock<Vec<Weak<Channel>>>
    //location: [f32; 2]
}

impl Token {
    pub fn clear_queue(&self) -> Vec<u8> {
        let mut lock = self.data.lock().unwrap();
        let mut buf = Vec::with_capacity(lock.len());
        buf.append(&mut lock);
        buf
    }

    pub fn enqueue(&self, buf: &[u8]) {
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

    pub fn rank(&self) -> u32 {
        1
    }

    pub fn join_channel(&self, channel: Weak<Channel>) {
        self.joined_channels.write().unwrap().push(channel)
    }

    pub fn joined_channels(&self) -> std::sync::RwLockReadGuard<Vec<Weak<Channel>>> {
        self.joined_channels.read().unwrap()
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
        (*other).id() == self.id
    }
}

// pub struct TokenList {
//     list: HashMap<String, Arc<Token>>
// }

impl List<Token> {
    pub fn add_token(&self, id: u32, name: String) -> Arc<Token> {
        let token = Uuid::new_v4().to_string();
        let token = Token {
            id, username: name,
            rank: 38, token,
            data: Mutex::new(Vec::new()),
            joined_channels: RwLock::new(Vec::new())
        };
        let token = Arc::new(token);
        self.insert(token.token(), token.clone());
        token
    }

    pub fn has_id(&self, id: u32) -> bool {
        for token in self.list.read().unwrap().values().map(|t| t.clone()) {
            if *token == id {
                return true;
            }
        }
        return false;
    }

    pub fn enqueue_all(&self, data: &[u8]) {
        self.list.read().unwrap().values().map(|t| t.clone()).for_each(|token| token.enqueue(data));
    }
}

// impl TokenList {
//     pub fn new() -> Self {
//         TokenList { list: HashMap::new() }
//     }

//     pub fn get(&self, token: String) -> Option<Arc<Token>> {
//         match self.list.get(&token) {
//             Some(token) => Some((*token).clone()),
//             None => None
//         }
//     }
// }