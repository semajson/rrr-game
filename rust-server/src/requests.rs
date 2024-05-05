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

pub fn process_request(request: String, db: Arc<impl Database>) -> String {
    let request = Request::new(request);

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

#[derive(Debug)]
struct Request {
    method: String,
    root: String,
    id: Option<String>,
    sub_root: Option<String>,
    headers: Vec<String>,
    body: Vec<String>,
}

impl Request {
    fn new(request: String) -> Option<Request> {
        let request = request
            .lines()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        // Todo - maybe make this more efficient?
        // e.g. pares the request line first, and it it doesn't match, then bail
        let mut request_line = "".to_string();
        let mut headers = vec![];
        let mut body = vec![];
        let mut in_body = false;
        for (index, line) in request.into_iter().enumerate() {
            if index == 0 {
                request_line = line;
            } else if line.is_empty() {
                in_body = true
            } else if in_body {
                body.push(line);
            } else {
                headers.push(line);
            }
        }

        // Parse request line
        let re = Regex::new(r"(?<method>GET|POST|DELETE) (?<root>/[a-z-]*)(?<id>/[a-z0-9]+)?(?<item>/[a-z0-9]+)? HTTP/1.1").unwrap();
        let cap = re.captures_iter(&request_line).last();

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

            let request = Request {
                method,
                root,
                id,
                sub_root,
                headers,
                body,
            };
            println!("{:?}", request);

            Some(request)
        } else {
            None
        }
    }
}
