use regex::{Match, Regex};
use std::{fs, sync::Arc};

use crate::{users, Database};

const GET: &str = "GET";
const POST: &str = "POST";
const DELETE: &str = "DELETE";
const USERS: &str = "/users";
const SESSIONS: &str = "/sessions";

#[derive(Debug)]
pub enum HttpErrorCode {
    Error404BadRequest,
    Error401Unauthorized,
    Error403Forbidden,
    Error404NotFround,
    Error409Conflict,
}

// struct Http

pub fn process_request(request: String, db: Arc<impl Database>) -> String {
    let request = Request::new(request);

    let (rsp_status, rsp_body) = if let Some(valid_request) = request {
        process_valid_request(valid_request, db)
    } else {
        // Maybe just drop this?
        (
            "HTTP/1.1 400 Bad request".to_string(),
            fs::read_to_string("404.html").unwrap(),
        )
    };

    let length = rsp_body.len();
    let response = format!("{rsp_status}\r\nContent-Length: {length}\r\n\r\n{rsp_body}");
    response
}

fn process_valid_request(valid_request: Request, db: Arc<impl Database>) -> (String, String) {
    let not_found = (
        "HTTP/1.1 404 NOT FOUND".to_string(),
        fs::read_to_string("404.html").unwrap(),
    );
    let not_implemented = ("HTTP/1.1 501 Not Implemented".to_string(), "".to_string());

    // Routes with no auth

    // Sessions
    if valid_request.root == SESSIONS && valid_request.item == None {
        if valid_request.method == POST {
            return not_implemented;
        }
    } else if valid_request.root == USERS && valid_request.item == None {
        if valid_request.method == POST {
            users::create_user(valid_request.body, db);

            return not_implemented;
        }
    }

    // Auth check

    // Routes with auth - if no matches here then return error

    // Sessions
    if valid_request.root == SESSIONS {
        if valid_request.method == DELETE {
            not_implemented
        } else {
            not_found
        }
    }
    // Users
    else if valid_request.root == USERS {
        if let Some(user_id) = valid_request.item {
            if valid_request.method == GET {
                // ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap())
                not_implemented
            } else if valid_request.method == POST {
                not_implemented
            } else if valid_request.method == DELETE {
                not_implemented
            } else {
                not_found
            }
        } else {
            not_found
        }
    } else {
        not_found
    }
}

#[derive(Debug)]
struct Request {
    method: String,
    root: String,
    id: Option<String>,
    item: Option<String>,
    headers: Vec<String>,
    body: String,
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
        let mut body = "".to_string();
        let mut in_body = false;
        for (index, line) in request.into_iter().enumerate() {
            if index == 0 {
                request_line = line;
            } else if line.is_empty() {
                in_body = true
            } else if in_body {
                body.push_str(&line);
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
            let item: Option<String> = match_to_string(valid_request.name("item"));

            let request = Request {
                method,
                root,
                id,
                item,
                headers,
                body,
            };

            Some(request)
        } else {
            None
        }
    }
}
