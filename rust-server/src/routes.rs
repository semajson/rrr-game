use crate::{
    http::{self, HttpError, HttpErrorCode, HttpMethod, Request, Response},
    jwt, rrr_game, users, Database,
};
use std::sync::Arc;

const USERS_ROUTE: &str = "/users";

const SESSIONS_ROUTE: &str = "/sessions";

const RRR_ROUTE: &str = "/rrr-game";
const RRR_PLAYERS_ROUTE: &str = "players";
const RRR_ACTIONS_ROUTE: &str = "actions";

pub fn process_request(request: String, db: Arc<impl Database>) -> String {
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

    let response = http::get_response(response);
    response.build_response()
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

    // Handle OPTION responses
    if valid_request.method == HttpMethod::OPTIONS {
        let allowed_headers = match (
            valid_request.resource.as_str(),
            valid_request.id,
            valid_request.sub_resource.as_deref(),
        ) {
            (SESSIONS_ROUTE, None, None) => Some(vec![HttpMethod::OPTIONS, HttpMethod::POST]),
            (USERS_ROUTE, None, None) => Some(vec![HttpMethod::OPTIONS, HttpMethod::POST]),
            (RRR_ROUTE, Some(_), Some(RRR_PLAYERS_ROUTE)) => Some(vec![
                HttpMethod::OPTIONS,
                HttpMethod::POST,
                HttpMethod::DELETE,
            ]),
            (_, _, _) => None,
        };

        return match allowed_headers {
            Some(headers) => Ok(Response {
                body: "".to_string(),
                headers: http::build_options_response_headers(headers),
                status: "200 OK".to_string(),
            }),
            None => not_found_error,
        };
    }

    //
    // Routes with no auth
    //

    // Sessions
    if valid_request.resource == SESSIONS_ROUTE && valid_request.id.is_none() {
        match valid_request.method {
            HttpMethod::POST => {
                return Response::response_from_body(users::login(valid_request.body, db))
            }
            _ => (),
        };
    }
    // Users
    else if valid_request.resource == USERS_ROUTE && valid_request.id.is_none() {
        match valid_request.method {
            HttpMethod::POST => {
                return Response::response_from_body(users::create_user(valid_request.body, db))
            }
            _ => (),
        }
    }

    //
    // Routes with auth
    //

    // Auth check
    let token = match valid_request.headers.get("Authorization") {
        Some(val) => match val.strip_prefix("Bearer ") {
            Some(token) => token,
            None => {
                return Err(HttpError {
                    code: HttpErrorCode::Error403Forbidden,
                    message: "You must be logged in.".to_string(),
                })
            }
        },
        None => {
            return Err(HttpError {
                code: HttpErrorCode::Error403Forbidden,
                message: "You must be logged in.".to_string(),
            })
        }
    };

    let username = jwt::validate_jwt(token, &"test".to_string())?;

    // Sessions
    if valid_request.resource == SESSIONS_ROUTE {
        // Could support DELETE, but that is unlikely to every happen as this is
        // fiddly for sessions.
        not_found_error
    }
    // Users
    else if valid_request.resource == USERS_ROUTE {
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
    else if valid_request.resource == RRR_ROUTE {
        if let Some(game_id) = valid_request.id {
            // Request for existing game

            // Todo, do I need to check user is in game?

            let sub_resource = valid_request.sub_resource.as_deref();
            match sub_resource {
                Some(RRR_ACTIONS_ROUTE) => match valid_request.method {
                    HttpMethod::POST => Response::response_from_body(rrr_game::do_action(
                        username,
                        valid_request.body,
                        valid_request.parameters,
                        game_id,
                        db,
                    )),
                    _ => not_found_error,
                },
                Some(RRR_PLAYERS_ROUTE) => {
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
