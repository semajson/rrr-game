use std::collections::HashMap;
use std::sync::Mutex;

pub struct Database {
    map: Mutex<HashMap<String, String>>, // todo - shoud this be string?
}
impl Database {
    pub fn new() -> Database {
        let mut map = HashMap::new();
        map.insert("test".to_string(), "1".to_string());
        let map = Mutex::new(map);
        Database { map }
    }

    pub fn get(&self, key: &str) -> String {
        // Todo - change this to be the proper get method
        let mut map = self.map.lock().unwrap();
        let value = map.get("test").unwrap().clone();
        let new_value = value + "1";
        map.insert("test".to_string(), new_value.clone());
        new_value
    }
    // Todo - implement update / create (unsure if want different ones)

    // Todo - implement remove
}
