use regex::{Captures, Match, Regex};
use std::{fs, sync::Arc, thread, time::Duration};

use crate::Database;

// No auth
// let (status_line, body) = match (method, root) {
//     "GET /test HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap()),
//     "POST /session HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap()),
//     "POST /users HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap()),
//     "GET /users/<User ID> HTTP/1.1" => {
//         ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap())
//     }
//     "DELETE /users/<User ID> HTTP/1.1" => {
//         ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap())
//     }

//     "POST /session HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap()),
//     _ => (
//         "HTTP/1.1 404 NOT FOUND",
//         fs::read_to_string("404.html").unwrap(),
//     ),
// };

pub fn process_request(request: Vec<String>, db: Arc<impl Database>) -> String {
    println!(" ");
    println!("Request is:");
    for line in request.iter() {
        println!("{:?}", line);
        if line.contains('\r') {
            println!("Has carrage return!");
        } else if line.contains('\n') {
            println!("Has a newline");
        }
    }
    println!(" ");

    let request = parse_reqeust(request);

    let (rsp_status_line, rsp_body) = if let Some(valid_request) = request {
        (
            "HTTP/1.1 404 NOT FOUND",
            fs::read_to_string("404.html").unwrap(),
        )
    } else {
        // Maybe just drop this?
        (
            "HTTP/1.1 404 NOT FOUND",
            fs::read_to_string("404.html").unwrap(),
        )
    };

    let length = rsp_body.len();
    let response = format!("{rsp_status_line}\r\nContent-Length: {length}\r\n\r\n{rsp_body}");
    response
}

struct Request {
    method: String,
    root: String,
    id: Option<String>,
    sub_root: Option<String>,
    body: Vec<String>,
}

fn parse_reqeust(request: Vec<String>) -> Option<Request> {
    println!("Request is:");
    for line in request.iter() {
        println!("{:?}", line);
        if line.contains('\r') {
            println!("Has carrage return!");
        } else if line.contains('\n') {
            println!("Has a newline");
        }
    }

    let re = Regex::new(r"(?<method>GET|POST|DELETE) (?<root>/[a-z-]*)(?<id>/[a-z0-9]+)?(?<item>/[a-z0-9]+)? HTTP/1.1").unwrap();
    let cap = re.captures_iter(&request[0]).last();

    if let Some(valid_request) = cap {
        fn match_to_string(option: Option<Match>) -> Option<String> {
            match option {
                Some(value) => Some(value.as_str().to_string()),
                None => None,
            }
        }

        // Method and root must be present if the regex is matched
        let method = valid_request.name("method").unwrap().as_str().to_string();
        let root = valid_request.name("root").unwrap().as_str().to_string();
        let id: Option<String> = match_to_string(valid_request.name("id"));
        let sub_root: Option<String> = match_to_string(valid_request.name("sub_root"));

        Some(Request {
            method,
            root,
            id,
            sub_root,
            body: request,
        })
    } else {
        None
    }
}
