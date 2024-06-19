use crate::{
    http::{HttpError, HttpErrorCode},
    rrr_game::{coord, create, CHUNK_LENGTH, GAME_NAME},
    Database,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
enum Move {
    North,
    East,
    South,
    West,
}

#[derive(Serialize, Deserialize)]
struct ActionRq {
    r#move: Move,
}

#[derive(Serialize, Deserialize)]
struct ActionRsp {
    user_coord: coord::UserCoord,
    top_left_visible_coord: coord::UserCoord,
}

pub fn do_action(
    username: String,
    body: String,
    parameters: Option<Vec<(String, String)>>,
    game_id: String,
    db: Arc<impl Database>,
) -> Result<String, HttpError> {
    // Parse the body
    let action: ActionRq = if let Ok(valid_body) = serde_json::from_str(&body) {
        valid_body
    } else {
        return Err(HttpError {
            code: HttpErrorCode::Error400BadRequest,
            message: "Request body has invalid format.".to_string(),
        });
    };

    // Workout the gamestate chunk coord
    let parameters = parameters.ok_or(HttpError {
        code: HttpErrorCode::Error400BadRequest,
        message: "User coords not supplied in POST action request.".to_string(),
    })?;
    let mut user_coord = coord::get_usercoord_from_params(parameters)?;
    let gamestate_coord = coord::user_coord_to_gamestate_coord(&user_coord, CHUNK_LENGTH);
    let top_left_visible_coord = coord::get_top_left_visible_coord(&gamestate_coord, CHUNK_LENGTH);

    // Get the gamestate chunk
    let gamestate_chunk = db
        .get(&(GAME_NAME.to_string() + ":" + &game_id + ":" + &gamestate_coord.id()))
        .unwrap(); // Todo error handling - goes for all unwrap() calls
    let mut gamestate_chunk: create::GamestateChunk =
        serde_json::from_str(&gamestate_chunk).unwrap();

    // Check user is in chunk
    if gamestate_chunk.users.contains_key(&username) {
        // Trust db over user supplied value
        user_coord = gamestate_chunk.users.get(&username).unwrap().clone();
    } else {
        // Possible todo - could add recovery code in here to search the neighbours
        return Err(HttpError {
            code: HttpErrorCode::Error500InternalServerError,
            message: "Can't find user in the gamestate chunk".to_string(),
        });
    }

    // Do move
    // x ->
    // y
    // |
    // v
    let new_user_coord = match action.r#move {
        Move::North => coord::UserCoord {
            x: user_coord.x,
            y: user_coord.y - 1,
        },
        Move::East => coord::UserCoord {
            x: user_coord.x + 1,
            y: user_coord.y,
        },
        Move::South => coord::UserCoord {
            x: user_coord.x,
            y: user_coord.y + 1,
        },
        Move::West => coord::UserCoord {
            x: user_coord.x - 1,
            y: user_coord.y,
        },
    };

    let new_relative_x = new_user_coord.x - (top_left_visible_coord.x + (CHUNK_LENGTH as i32));
    let new_relative_y = new_user_coord.y - (top_left_visible_coord.y + (CHUNK_LENGTH as i32));
    assert!(new_relative_x > 0);
    assert!(new_relative_y > 0);
    let new_relative_x = new_relative_x as usize;
    let new_relative_y = new_relative_y as usize;
    // Todo - deal with going off edge of chunk, requires updating top_left_visible_coord
    let rsp = if gamestate_chunk.terrain[new_relative_y][new_relative_x] == create::TILE_GRASS {
        // Move is valid

        gamestate_chunk
            .users
            .insert(username, new_user_coord.clone());

        // Write to DB
        db.set(
            GAME_NAME.to_string() + ":" + &game_id + ":" + &gamestate_chunk.get_id(),
            serde_json::to_string(&gamestate_chunk).unwrap(),
        );

        // Return
        // Todo - workout if want to return the gamestate here...
        ActionRsp {
            user_coord: new_user_coord,
            top_left_visible_coord,
        }
    } else {
        // Move is invalid
        ActionRsp {
            user_coord,
            top_left_visible_coord,
        }
    };
    Ok(serde_json::to_string(&rsp).unwrap())
}
