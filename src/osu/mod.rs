use std::sync::{RwLock, Arc};
use std::collections::HashMap;

pub mod packets;
pub mod token;
pub mod channel;
pub mod matches;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, enumn::N)]
pub enum GameMode { STANDARD, TAIKO, CTB, MANIA }

#[derive(Debug, Default)]
pub struct List<V> {
    list: RwLock<HashMap<String, Arc<V>>>
}

impl<V> List<V> {
    pub fn new() -> Self {
        List {
            list: RwLock::new(HashMap::new())
        }
    }

    pub fn get(&self, key: &str) -> Option<Arc<V>> {
        match self.list.read().unwrap().get(key) {
            Some(val) => Some((*val).clone()),
            None => None
        }
    }

    pub fn find<F: FnMut(&&Arc<V>) -> bool>(&self, fun: F) -> Option<Arc<V>> {
        match self.list.read().unwrap().values().find(fun) {
            Some(val) => Some((*val).clone()),
            None => None
        }
    }

    pub fn entries(&self) -> Vec<Arc<V>> {
        self.list.read().unwrap().values().cloned().collect()
    }

    pub fn remove(&self, key: &str) -> Option<Arc<V>> {
        self.list.write().unwrap().remove(key)
    }

    fn insert(&self, key: String, val: Arc<V>) {
        self.list.write().unwrap().insert(key, val);
    }
}