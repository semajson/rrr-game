use crate::{
    http::{self, HttpError, HttpErrorCode, HttpMethod, Response},
    jwt, rrr_game, users, Database,
};
use std::sync::Arc;

const USERS: &str = "/users";
const SESSIONS: &str = "/sessions";
const RRR_GAME: &str = "/rrr-game";

const RRR_GAME_PLAYERS: &str = "players";
const RRR_GAME_ACTIONS: &str = "actions";

pub fn process_request(request: String, db: Arc<impl Database>) -> String {
    println!("{:?}\n ", request); // todo logging
    let request = http::Request::new(request);

    let response = if let Some(valid_request) = request {
        process_valid_request(valid_request, db)
    } else {
        // The request wasn't valid
        // Maybe just drop this?
        Err(HttpError {
            code: HttpErrorCode::Error400BadRequest,
            message: "Request is not a valid HTTP request".to_string(),
        })
    };

    http::build_response(response)
}

fn process_valid_request(
    valid_request: http::Request,
    db: Arc<impl Database>,
) -> Result<Response, HttpError> {
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
        match valid_request.method {
            HttpMethod::POST => {
                return Response::response_from_body(users::login(valid_request.body, db))
            }
            HttpMethod::OPTIONS => {
                return Ok(Response {
                    body: "".to_string(),
                    headers: http::build_options_response_headers(vec![
                        HttpMethod::OPTIONS,
                        HttpMethod::POST,
                    ]),
                    status: "200 OK".to_string(),
                });
            }
            _ => (),
        };
    } else if valid_request.resource == USERS && valid_request.id.is_none() {
        match valid_request.method {
            HttpMethod::POST => {
                return Response::response_from_body(users::create_user(valid_request.body, db))
            }
            HttpMethod::OPTIONS => {
                return Ok(Response {
                    body: "".to_string(),
                    headers: http::build_options_response_headers(vec![
                        HttpMethod::OPTIONS,
                        HttpMethod::POST,
                    ]),
                    status: "200 OK".to_string(),
                });
            }

            _ => (),
        };
    }

    //
    // Routes with auth
    //

    // Auth check
    let token = match valid_request.headers.get("Authorization") {
        Some(val) => val,
        None => {
            return Err(HttpError {
                code: HttpErrorCode::Error403Forbidden,
                message: "You must be logged in.".to_string(),
            })
        }
    };

    let username = jwt::validate_jwt(token, &"test".to_string())?;

    // Sessions
    if valid_request.resource == SESSIONS {
        // Could support DELETE, but that is unlikely to every happen as this is
        // fiddly for sessions.
        not_found_error
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
            match valid_request.method {
                HttpMethod::GET => Response::response_from_body(users::get_user(username, db)),
                _ => not_implemented_error,
            }
        } else {
            not_found_error
        }
    }
    // RRR game
    else if valid_request.resource == RRR_GAME {
        if let Some(game_id) = valid_request.id {
            // Request for existing game

            // Todo, do I need to check user is in game?

            let sub_resource = valid_request.sub_resource.as_deref();
            match sub_resource {
                Some(RRR_GAME_ACTIONS) => match valid_request.method {
                    HttpMethod::POST => Response::response_from_body(rrr_game::do_action(
                        username,
                        valid_request.body,
                        valid_request.parameters,
                        game_id,
                        db,
                    )),
                    _ => not_found_error,
                },
                Some(RRR_GAME_PLAYERS) => {
                    match valid_request.method {
                        HttpMethod::POST => {
                            // join game
                            not_implemented_error
                        }
                        HttpMethod::DELETE => {
                            // leave game
                            not_implemented_error
                        }
                        _ => not_found_error,
                    }
                }
                Some(_) => not_found_error,
                None => {
                    match valid_request.method {
                        HttpMethod::GET => Response::response_from_body(rrr_game::get_gamestate(
                            username,
                            valid_request.parameters,
                            game_id,
                            db,
                        )),
                        HttpMethod::DELETE => {
                            // Delete the game
                            not_implemented_error
                        }
                        _ => not_found_error,
                    }
                }
            }
        } else {
            // No game_id specified
            match valid_request.method {
                HttpMethod::POST => {
                    Response::response_from_body(rrr_game::create_game(username, db))
                }
                HttpMethod::GET => {
                    // Get list of available games - probably won't do
                    not_implemented_error
                }
                _ => not_found_error,
            }
        }
    } else {
        // Unknown resource
        not_found_error
    }
}
