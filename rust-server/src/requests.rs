use crate::{users, Database};
use regex::{Match, Regex};
use serde::{Deserialize, Serialize};
use std::{fs, sync::Arc};

const GET: &str = "GET";
const POST: &str = "POST";
const DELETE: &str = "DELETE";
const USERS: &str = "/users";
const SESSIONS: &str = "/sessions";

#[derive(Debug)]
pub enum HttpErrorCode {
    Error400BadRequest,
    Error401Unauthorized,
    Error403Forbidden,
    Error404NotFround,
    Error409Conflict,
    Error501NotImplemented,
}

#[derive(Debug)]
pub struct HttpError {
    pub code: HttpErrorCode,
    pub message: String,
}

pub fn process_request(request: String, db: Arc<impl Database>) -> String {
    let request = Request::new(request);

    let response = if let Some(valid_request) = request {
        process_valid_request(valid_request, db)
    } else {
        // Maybe just drop this?
        Err(HttpError {
            code: HttpErrorCode::Error400BadRequest,
            message: "Request is not a valid HTTP request".to_string(),
        })
    };

    let (rsp_status, rsp_body) = match response {
        Ok(body) => ("200 OK", body),
        Err(error) => {
            let error_body = error_body(error.message);
            match error.code {
                HttpErrorCode::Error400BadRequest => ("400 Bad request", error_body),
                HttpErrorCode::Error401Unauthorized => ("401 Unauthorized", error_body),
                HttpErrorCode::Error403Forbidden => ("403 Forbidden", error_body),
                HttpErrorCode::Error404NotFround => ("404 Not found", error_body),
                HttpErrorCode::Error409Conflict => ("409 Conflict", error_body),
                HttpErrorCode::Error501NotImplemented => ("501 Not implemented", error_body),
            }
        }
    };

    let length = rsp_body.len();
    let response = format!("HTTP/1.1 {rsp_status}\r\nContent-Length: {length}\r\n\r\n{rsp_body}");
    response
}

#[derive(Serialize, Deserialize)]
struct ErrorMsg {
    error_message: String,
}

fn error_body(error_message: String) -> String {
    let error_body = ErrorMsg { error_message };
    serde_json::to_string(&error_body).unwrap()
}

fn process_valid_request(
    valid_request: Request,
    db: Arc<impl Database>,
) -> Result<String, HttpError> {
    let not_found_error = Err(HttpError {
        code: HttpErrorCode::Error404NotFround,
        message: "Route not found".to_string(),
    });

    let not_implemented_error = Err(HttpError {
        code: HttpErrorCode::Error501NotImplemented,
        message: "Method not implemented for route".to_string(),
    });

    // Routes with no auth

    // Sessions
    if valid_request.root == SESSIONS && valid_request.item == None {
        if valid_request.method == POST {
            return users::login(valid_request.body, db);
        }
    } else if valid_request.root == USERS && valid_request.item == None {
        if valid_request.method == POST {
            return users::create_user(valid_request.body, db);
        }
    }

    // Auth check

    // Routes with auth - if no matches here then return error

    // Sessions
    if valid_request.root == SESSIONS {
        if valid_request.method == DELETE {
            not_implemented_error
        } else {
            not_found_error
        }
    }
    // Users
    else if valid_request.root == USERS {
        if let Some(user_id) = valid_request.item {
            if valid_request.method == GET {
                // ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap())
                not_implemented_error
            } else if valid_request.method == POST {
                not_implemented_error
            } else if valid_request.method == DELETE {
                not_implemented_error
            } else {
                not_found_error
            }
        } else {
            not_found_error
        }
    } else {
        not_found_error
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
