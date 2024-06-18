use crate::{
    requests::{HttpError, HttpErrorCode},
    users, Database,
};
use rand::{distributions::Alphanumeric, Rng}; // 0.8
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
pub struct GamestateCoord {
    pub x: i32,
    pub y: i32,
}
impl GamestateCoord {
    pub fn id(&self) -> String {
        self.x.to_string() + "-" + &self.y.to_string()
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
// (0,0) user coord cooresponds to centre of the (0,0) gamestate chunk.
// Gamestate chunk side length must always be odd, to ensure a centre
// user coord position exists.
pub struct UserCoord {
    pub x: i32,
    pub y: i32,
}

pub fn user_coord_to_gamestate_coord(
    user_coord: &UserCoord,
    chunk_length: usize,
) -> GamestateCoord {
    let chunk_length = chunk_length as i32;
    assert!((chunk_length % 2) == 1);
    let offset = (chunk_length - 1) / 2;

    // Might be a nicer way to do this
    // Have a lot of UTs, so can easily refactor!
    let chunk_x = if user_coord.x > 0 {
        (user_coord.x + offset) / chunk_length
    } else {
        (user_coord.x - offset) / chunk_length
    };
    let chunk_y = if user_coord.y > 0 {
        (user_coord.y + offset) / chunk_length
    } else {
        (user_coord.y - offset) / chunk_length
    };

    GamestateCoord {
        x: chunk_x,
        y: chunk_y,
    }
}

pub fn get_top_left_visible_coord(
    gamestate_coord: &GamestateCoord,
    chunk_length: usize,
) -> UserCoord {
    let chunk_length = chunk_length as i32;
    assert!((chunk_length % 2) == 1);

    // Assuming visible == 3*3 chunks
    let top_left_chunk = GamestateCoord {
        x: gamestate_coord.x - 1,
        y: gamestate_coord.y - 1,
    };

    let diff_to_edge = chunk_length / 2;
    let top_left_x = (top_left_chunk.x * chunk_length) - diff_to_edge;
    let top_left_y = (top_left_chunk.y * chunk_length) - diff_to_edge;

    UserCoord {
        x: top_left_x,
        y: top_left_y,
    }
}

pub fn get_usercoord_from_params(
    parameters: Vec<(String, String)>,
) -> Result<UserCoord, HttpError> {
    fn get_int_param(key: &str, parameters: &Vec<(String, String)>) -> Option<i32> {
        let found_parameters = parameters
            .iter()
            .filter(|(k, _v)| key == k)
            .map(|(_k, v)| v.clone())
            .collect::<Vec<String>>();

        if found_parameters.is_empty() {
            None
        } else {
            found_parameters[0].parse::<i32>().ok()
        }
    }
    let found_x = get_int_param("x", &parameters).ok_or(HttpError {
        code: HttpErrorCode::Error400BadRequest,
        message: "X position missing or invalid.".to_string(),
    })?;
    let found_y = get_int_param("y", &parameters).ok_or(HttpError {
        code: HttpErrorCode::Error400BadRequest,
        message: "Y position missing or invalid.".to_string(),
    })?;
    let user_coord = UserCoord {
        x: found_x,
        y: found_y,
    };
    Ok(user_coord)
}

#[test]
fn test_user_coord_to_gamestate_coord() {
    assert_eq!(
        user_coord_to_gamestate_coord(&UserCoord { x: 0, y: 0 }, 9),
        GamestateCoord { x: 0, y: 0 }
    );

    assert_eq!(
        user_coord_to_gamestate_coord(&UserCoord { x: -7, y: -8 }, 9),
        GamestateCoord { x: -1, y: -1 }
    );
    assert_eq!(
        user_coord_to_gamestate_coord(&UserCoord { x: 0, y: -5 }, 9),
        GamestateCoord { x: 0, y: -1 }
    );
    assert_eq!(
        user_coord_to_gamestate_coord(&UserCoord { x: 5, y: -13 }, 9),
        GamestateCoord { x: 1, y: -1 }
    );

    assert_eq!(
        user_coord_to_gamestate_coord(&UserCoord { x: -5, y: 0 }, 9),
        GamestateCoord { x: -1, y: 0 }
    );
    assert_eq!(
        user_coord_to_gamestate_coord(&UserCoord { x: 4, y: -4 }, 9),
        GamestateCoord { x: 0, y: 0 }
    );
    assert_eq!(
        user_coord_to_gamestate_coord(&UserCoord { x: 13, y: -3 }, 9),
        GamestateCoord { x: 1, y: 0 }
    );

    assert_eq!(
        user_coord_to_gamestate_coord(&UserCoord { x: -13, y: 6 }, 9),
        GamestateCoord { x: -1, y: 1 }
    );
    assert_eq!(
        user_coord_to_gamestate_coord(&UserCoord { x: 1, y: 10 }, 9),
        GamestateCoord { x: 0, y: 1 }
    );
    assert_eq!(
        user_coord_to_gamestate_coord(&UserCoord { x: 5, y: 9 }, 9),
        GamestateCoord { x: 1, y: 1 }
    );

    assert_eq!(
        user_coord_to_gamestate_coord(&UserCoord { x: -14, y: -14 }, 9),
        GamestateCoord { x: -2, y: -2 }
    );
}

#[test]
fn test_get_top_left_visible_coord() {
    assert_eq!(
        get_top_left_visible_coord(&GamestateCoord { x: 0, y: 0 }, 9),
        UserCoord { x: -13, y: -13 }
    );
    assert_eq!(
        get_top_left_visible_coord(&GamestateCoord { x: 1, y: 1 }, 9),
        UserCoord { x: -4, y: -4 }
    );
    assert_eq!(
        get_top_left_visible_coord(&GamestateCoord { x: 1, y: 0 }, 9),
        UserCoord { x: -4, y: -13 }
    );
    assert_eq!(
        get_top_left_visible_coord(&GamestateCoord { x: -1, y: -1 }, 9),
        UserCoord { x: -22, y: -22 }
    );
}
