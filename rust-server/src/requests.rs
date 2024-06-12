use crate::{jwt, rrr_game, users, Database};
use regex::{Match, Regex};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

const GET: &str = "GET";
const POST: &str = "POST";
const DELETE: &str = "DELETE";
const USERS: &str = "/users";
const SESSIONS: &str = "/sessions";
const RRR_GAME: &str = "/rrr-game";

#[derive(Debug)]
pub enum HttpErrorCode {
    Error400BadRequest,
    Error401Unauthorized,
    Error403Forbidden,
    Error404NotFround,
    Error409Conflict,
    Error500InternalServerError,
    Error501NotImplemented,
    Error503ServiceUnavailable,
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
                HttpErrorCode::Error503ServiceUnavailable => {
                    ("503 Service unavailable", error_body)
                }
                HttpErrorCode::Error500InternalServerError => {
                    ("500 Internal Server Error", error_body)
                }
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

    //
    // Routes with no auth
    //

    // Sessions
    if valid_request.resource == SESSIONS && valid_request.id.is_none() {
        if valid_request.method == POST {
            return users::login(valid_request.body, db);
        }
    } else if valid_request.resource == USERS && valid_request.id.is_none() {
        if valid_request.method == POST {
            return users::create_user(valid_request.body, db);
        }
    }

    //
    // Routes with auth
    //

    // Auth check
    let token = valid_request.headers.get("Authorization").unwrap(); // todo error handling
    let username = jwt::validate_jwt(token, &"test".to_string())?;

    // Sessions
    if valid_request.resource == SESSIONS {
        if valid_request.method == DELETE {
            // Probably will never implement, as for jwt this is a pain
            not_implemented_error
        } else {
            not_found_error
        }
    }
    // Users
    else if valid_request.resource == USERS {
        if let Some(user_id) = valid_request.id {
            if user_id != username {
                return Err(HttpError {
                    code: HttpErrorCode::Error403Forbidden,
                    message: "You are not authorized to access this user.".to_string(),
                });
            }

            if valid_request.method == GET {
                users::get_user(username, db)
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
    }
    // RRR game
    else if valid_request.resource == RRR_GAME {
        if let Some(game_id) = valid_request.id {
            // Request for existing game

            if let Some(sub_resource) = valid_request.sub_resource.clone() {
                if sub_resource == "moves" {
                    if valid_request.method == POST {
                        // Make a move
                        not_implemented_error
                    } else {
                        not_found_error
                    }
                } else if sub_resource == "players" {
                    if valid_request.method == POST {
                        // Join game
                        not_implemented_error
                    } else if valid_request.method == DELETE {
                        // Leave game
                        not_implemented_error
                    } else {
                        not_found_error
                    }
                } else {
                    not_found_error
                }
            } else if valid_request.method == GET {
                // Get the gamestate
                rrr_game::get_gamestate(username, valid_request.parameters, game_id, db)
            } else if valid_request.method == DELETE {
                // Delete the game
                not_implemented_error
            } else {
                not_found_error
            }
        } else if valid_request.method == POST {
            // Create game
            rrr_game::create_game(username, db)
        } else if valid_request.method == GET {
            // Get list of available games - probably won't do
            not_implemented_error
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
    resource: String,
    id: Option<String>,
    sub_resource: Option<String>,
    headers: HashMap<String, String>,
    body: String,
    parameters: Option<Vec<(String, String)>>,
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
        let mut headers = HashMap::new();
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
                // Todo = error parsing?
                let (header, value) = line.split_once(": ").unwrap();
                headers.insert(header.to_string(), value.to_string());
            }
        }

        // Parse request line
        let re = Regex::new(r"(?<method>GET|POST|DELETE) (?<resource>/[a-z-]*)(?<id>/[a-zA-Z0-9]+)?(?<sub_resource>/[a-z0-9]+)?(?<parameters>\?[a-z-0-9=&]+)? HTTP/1.1").unwrap();
        let cap = re.captures_iter(&request_line).last();

        if let Some(valid_request) = cap {
            fn match_to_string_no_leading_char(option: Option<Match>) -> Option<String> {
                option.map(|value| value.as_str()[1..].to_string())
            }

            // Method and root must be present if the regex is matched
            let method = valid_request.name("method").unwrap().as_str().to_string();
            let resource = valid_request.name("resource").unwrap().as_str().to_string();
            let id: Option<String> = match_to_string_no_leading_char(valid_request.name("id"));
            let sub_resource: Option<String> =
                match_to_string_no_leading_char(valid_request.name("sub_resource"));
            let parameters: Option<String> =
                match_to_string_no_leading_char(valid_request.name("parameters"));
            let parameters = match parameters {
                Some(raw) => {
                    let params = raw.split('&').map(String::from).collect::<Vec<String>>();

                    let params = params
                        .into_iter()
                        .map(|x| x.split('=').map(String::from).collect::<Vec<String>>())
                        .filter(|param| param.len() == 2) // This means invalid params are ignored
                        .map(|param| (param[0].clone(), param[1].clone()))
                        .collect();

                    Some(params)
                }
                None => None,
            };

            let request = Request {
                method,
                resource,
                id,
                sub_resource,
                headers,
                body,
                parameters,
            };

            Some(request)
        } else {
            None
        }
    }
}
