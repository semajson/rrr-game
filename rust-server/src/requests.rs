use std::{fs, sync::Arc, thread, time::Duration};

use crate::Database;

pub fn process_request(request: Vec<String>, db: Arc<Database>) -> String {
    let (status_line, filename) = match request[0].as_ref() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let value = db.get("test");
    println!("Database value is {:}", value);
    db.set("test".to_string(), value + "1");

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    response
}
