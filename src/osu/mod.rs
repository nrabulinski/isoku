use std::borrow::Borrow;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

pub mod channel;
pub mod matches;
pub mod packets;
pub mod token;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, enumn::N)]
pub enum GameMode {
    STANDARD,
    TAIKO,
    CTB,
    MANIA,
}

#[derive(Debug, Default)]
pub struct List<K, V>
where
    K: Hash + Eq,
{
    list: RwLock<HashMap<K, Arc<V>>>,
}

impl<K, V> List<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        List {
            list: RwLock::new(HashMap::new()),
        }
    }

    pub fn get<T>(&self, key: &T) -> Option<Arc<V>>
    where
        K: Borrow<T>,
        T: Hash + Eq + ?Sized,
    {
        match self.list.read().unwrap().get(key) {
            Some(val) => Some((*val).clone()),
            None => None,
        }
    }

    pub fn find<F: FnMut(&&Arc<V>) -> bool>(&self, fun: F) -> Option<Arc<V>> {
        match self.list.read().unwrap().values().find(fun) {
            Some(val) => Some((*val).clone()),
            None => None,
        }
    }

    pub fn entries(&self) -> Vec<Arc<V>> {
        self.list.read().unwrap().values().cloned().collect()
    }

    pub fn remove<T>(&self, key: &T) -> Option<Arc<V>>
    where
        K: Borrow<T>,
        T: Hash + Eq + ?Sized,
    {
        self.list.write().unwrap().remove(key)
    }

    fn insert(&self, key: K, val: Arc<V>) {
        self.list.write().unwrap().insert(key, val);
    }
}
