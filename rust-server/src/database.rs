use std::collections::HashMap;
use std::sync::Mutex;

pub struct Database {
    map: Mutex<HashMap<String, String>>, // todo - shoud this be string?
}
impl Database {
    pub fn new() -> Database {
        let map = HashMap::new();
        let map = Mutex::new(map);
        Database { map }
    }

    pub fn get(&self, key: &str) -> String {
        let map = self.map.lock().unwrap();
        // Todo - handle the unwrap better.
        // probaly return option and have the calling code deal with the error as required
        map.get(key).unwrap().clone()
    }

    pub fn set(&self, key: String, value: String) {
        let mut map = self.map.lock().unwrap();
        map.insert(key, value);
    }

    pub fn del(&self, key: &str) {
        let mut map = self.map.lock().unwrap();
        map.remove(key);
    }
}
