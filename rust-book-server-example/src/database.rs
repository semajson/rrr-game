use std::collections::HashMap;

pub struct Database {
    map: HashMap<String, String>,
    // todo - could this be a mutex<hashmap>?  https://docs.rs/dashmap/latest/dashmap/struct.DashMap.html says:
    //  This allows you to put a DashMap in an Arc<T> and share it between threads while being able to modify it.
    // todo - shoud this be string?
}
impl Database {
    pub fn new() -> Database {
        let mut map = HashMap::new();
        map.insert("test".to_string(), "1".to_string());
        // let map = Mutex::new(map);
        Database { map }
    }

    pub fn get(&mut self, key: &str) -> String {
        // Todo - change this to be the proper get method
        let value = self.map.get("test").unwrap().clone();
        let new_value = value + "1";
        self.map.insert("test".to_string(), new_value.clone());
        new_value
    }
    // Todo - implement update / create (unsure if want different ones)

    // Todo - implement remove
}
