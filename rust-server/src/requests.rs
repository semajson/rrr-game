use std::{fs, sync::Arc, thread, time::Duration};

use crate::Database;

pub fn process_request(request: Vec<String>, db: Arc<impl Database>) -> String {
    println!("Request is:");
    for line in request.iter() {
        println!("{:?}", line);
    }

    // No auth
    let (status_line, body) = match request[0].as_ref() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap()),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap())
        }
        _ => (
            "HTTP/1.1 404 NOT FOUND",
            fs::read_to_string("404.html").unwrap(),
        ),
    };

    let length = body.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{body}");
    response
}
