use std::sync::{RwLock, Arc};
use std::collections::HashMap;

pub mod packets;
pub mod token;
pub mod channel;

#[derive(Debug)]
pub struct List<T> {
    list: RwLock<HashMap<String, Arc<T>>>
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            list: RwLock::new(HashMap::new())
        }
    }

    pub fn get(&self, key: &str) -> Option<Arc<T>> {
        match self.list.read().unwrap().get(key) {
            Some(val) => Some((*val).clone()),
            None => None
        }
    }

    pub fn find<F: FnMut(&&Arc<T>) -> bool>(&self, fun: F) -> Option<Arc<T>> {
        match self.list.read().unwrap().values().find(fun) {
            Some(val) => Some((*val).clone()),
            None => None
        }
    }

    pub fn entries(&self) -> Vec<Arc<T>> {
        self.list.read().unwrap().values().map(|t| t.clone()).collect()
    }

    pub fn remove(&self, key: &str) -> Option<Arc<T>> {
        self.list.write().unwrap().remove(key)
    }

    fn insert(&self, key: String, val: Arc<T>) {
        self.list.write().unwrap().insert(key, val);
    }
}