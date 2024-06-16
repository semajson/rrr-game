use std::collections::HashMap;
use std::sync::Mutex;

pub trait Database {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: String, value: String);
    fn del(&self, key: &str);
}

pub struct LocalDatabase {
    map: Mutex<HashMap<String, String>>,
}
impl LocalDatabase {
    pub fn new() -> LocalDatabase {
        let map = HashMap::new();
        let map = Mutex::new(map);
        LocalDatabase { map }
    }
}
impl Database for LocalDatabase {
    fn get(&self, key: &str) -> Option<String> {
        let map = self.map.lock().unwrap();
        // Todo - handle the unwrap better.
        // probaly return option and have the calling code deal with the error as required
        map.get(key).cloned()
    }

    fn set(&self, key: String, value: String) {
        let mut map = self.map.lock().unwrap();
        map.insert(key, value);
    }

    fn del(&self, key: &str) {
        let mut map = self.map.lock().unwrap();
        map.remove(key);
    }
}

impl Default for LocalDatabase {
    fn default() -> Self {
        Self::new()
    }
}
