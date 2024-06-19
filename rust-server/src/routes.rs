use crate::{
    http,
    http::{HttpError, HttpErrorCode},
    jwt, rrr_game, users, Database,
};
use std::sync::Arc;

const USERS: &str = "/users";
const SESSIONS: &str = "/sessions";
const RRR_GAME: &str = "/rrr-game";

pub fn process_request(request: String, db: Arc<impl Database>) -> String {
    let request = http::Request::new(request);

    let response_body = if let Some(valid_request) = request {
        process_valid_request(valid_request, db)
    } else {
        // The request wasn't valid
        // Maybe just drop this?
        Err(HttpError {
            code: HttpErrorCode::Error400BadRequest,
            message: "Request is not a valid HTTP request".to_string(),
        })
    };

    http::build_response(response_body)
}

fn process_valid_request(
    valid_request: http::Request,
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
        if valid_request.method == http::POST {
            return users::login(valid_request.body, db);
        }
    } else if valid_request.resource == USERS && valid_request.id.is_none() {
        if valid_request.method == http::POST {
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
        if valid_request.method == http::DELETE {
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

            if valid_request.method == http::GET {
                users::get_user(username, db)
            } else if valid_request.method == http::POST {
                not_implemented_error
            } else if valid_request.method == http::DELETE {
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
                if sub_resource == "actions" {
                    if valid_request.method == http::POST {
                        // Make a move
                        rrr_game::do_action(
                            username,
                            valid_request.body,
                            valid_request.parameters,
                            game_id,
                            db,
                        )
                    } else {
                        not_found_error
                    }
                } else if sub_resource == "players" {
                    if valid_request.method == http::POST {
                        // Join game
                        not_implemented_error
                    } else if valid_request.method == http::DELETE {
                        // Leave game
                        not_implemented_error
                    } else {
                        not_found_error
                    }
                } else {
                    not_found_error
                }
            } else if valid_request.method == http::GET {
                // Get the gamestate
                rrr_game::get_gamestate(username, valid_request.parameters, game_id, db)
            } else if valid_request.method == http::DELETE {
                // Delete the game
                not_implemented_error
            } else {
                not_found_error
            }
        } else if valid_request.method == http::POST {
            // Create game
            rrr_game::create_game(username, db)
        } else if valid_request.method == http::GET {
            // Get list of available games - probably won't do
            not_implemented_error
        } else {
            not_found_error
        }
    } else {
        not_found_error
    }
}
