use crate::{
    http::{HttpError, HttpErrorCode},
    rrr_game::{coord, create, CHUNK_LENGTH, GAME_NAME},
    Database,
};

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub fn get_visible_gamestate(
    user_coord: &coord::UserCoord,
    username: String,
    game_id: &String,
    db: Arc<impl Database>,
) -> Result<VisibleGamestate, HttpError> {
    let centre_gamestate_coord = coord::user_coord_to_gamestate_coord(user_coord, CHUNK_LENGTH);

    // Get centre create::GamestateChunk
    let centre_gamestate_chunk = db
        .get(&(GAME_NAME.to_string() + ":" + game_id + ":" + &centre_gamestate_coord.id()))
        .unwrap(); // Todo error handling - goes for all unwrap() calls
    let centre_gamestate_chunk: create::GamestateChunk =
        serde_json::from_str(&centre_gamestate_chunk).unwrap();

    // Check user is in chunk
    if !centre_gamestate_chunk.users.contains_key(&username) {
        // Possible todo - could add recovery code in here to search the neighbours
        return Err(HttpError {
            code: HttpErrorCode::Error500InternalServerError,
            message: "Can't find user in the gamestate chunk".to_string(),
        });
    }

    // Get neighbours gamestate
    let neighbours = centre_gamestate_chunk.get_neighbours();
    let mut chunks = HashMap::from([(centre_gamestate_coord.clone(), centre_gamestate_chunk)]);
    for neighbour in neighbours {
        let neighbour_gamestate_chunk = db
            .get(&(GAME_NAME.to_string() + ":" + game_id + ":" + &neighbour.id()))
            .unwrap();
        let neighbour_gamestate_chunk = serde_json::from_str(&neighbour_gamestate_chunk).unwrap();
        chunks.insert(
            neighbour.clone(),
            neighbour_gamestate_chunk, // Todo error handling - goes for all unwrap() calls,
        );
    }

    // Return visible gamestate
    Ok(create_visible_gamestate(centre_gamestate_coord, chunks))
}

pub fn get_gamestate(
    username: String,
    parameters: Option<Vec<(String, String)>>,
    game_id: String,
    db: Arc<impl Database>,
) -> Result<String, HttpError> {
    // Convert the passed in params to coord::coord::UserCoord
    let parameters = parameters.ok_or(HttpError {
        code: HttpErrorCode::Error400BadRequest,
        message: "User coords not supplied in GET gamestate request.".to_string(),
    })?;
    let user_coord = coord::get_usercoord_from_params(parameters)?;

    let visible_gamestate = get_visible_gamestate(&user_coord, username, &game_id, db)?;
    Ok(serde_json::to_string(&visible_gamestate).unwrap())
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct VisibleGamestate {
    terrain: Vec<Vec<char>>,
    users: HashMap<String, coord::UserCoord>,
}

fn create_visible_gamestate(
    centre: coord::GamestateCoord,
    chunks: HashMap<coord::GamestateCoord, create::GamestateChunk>,
) -> VisibleGamestate {
    // Get users
    let mut users: HashMap<String, coord::UserCoord> = HashMap::new();
    for (_, chunk) in chunks.iter() {
        users.extend(chunk.users.clone());
    }

    // Userful inline functions (?)
    let get_chunk = |dx, dy| {
        chunks
            .get(&coord::GamestateCoord {
                x: centre.x + dx,
                y: centre.y + dy,
            })
            .unwrap()
    };
    let get_new_rows = |dy| {
        let mut rows = vec![];
        let left = get_chunk(-1, dy).terrain.clone();
        let middle = get_chunk(0, dy).terrain.clone();
        let right = get_chunk(1, dy).terrain.clone();
        for (i, _) in left.iter().enumerate() {
            let mut new_row = vec![];
            new_row.extend(left[i].clone());
            new_row.extend(middle[i].clone());
            new_row.extend(right[i].clone());

            rows.push(new_row);
        }
        rows
    };

    // Get terrain

    let mut terrain: Vec<Vec<char>> = vec![];

    // Top
    terrain.extend(get_new_rows(-1));

    // Middle
    terrain.extend(get_new_rows(0));

    // Bottom
    terrain.extend(get_new_rows(1));

    VisibleGamestate { terrain, users }
}

#[test]
fn test_create_visible_gamestate() {
    let chunks: HashMap<coord::GamestateCoord, create::GamestateChunk> = HashMap::from([
        (
            coord::GamestateCoord { x: 9, y: 9 },
            create::GamestateChunk {
                coord: coord::GamestateCoord { x: 9, y: 9 },
                terrain: vec![vec!['a', 'b'], vec!['A', 'B']],
                users: HashMap::new(),
            },
        ),
        (
            coord::GamestateCoord { x: 10, y: 9 },
            create::GamestateChunk {
                coord: coord::GamestateCoord { x: 10, y: 9 },
                terrain: vec![vec!['c', 'd'], vec!['C', 'D']],
                users: HashMap::new(),
            },
        ),
        (
            coord::GamestateCoord { x: 11, y: 9 },
            create::GamestateChunk {
                coord: coord::GamestateCoord { x: 11, y: 9 },
                terrain: vec![vec!['e', 'f'], vec!['E', 'F']],
                users: HashMap::new(),
            },
        ),
        (
            coord::GamestateCoord { x: 9, y: 10 },
            create::GamestateChunk {
                coord: coord::GamestateCoord { x: 9, y: 19 },
                terrain: vec![vec!['h', 'i'], vec!['H', 'I']],
                users: HashMap::new(),
            },
        ),
        (
            coord::GamestateCoord { x: 10, y: 10 },
            create::GamestateChunk {
                coord: coord::GamestateCoord { x: 10, y: 10 },
                terrain: vec![vec!['j', 'k'], vec!['J', 'K']],
                users: HashMap::new(),
            },
        ),
        (
            coord::GamestateCoord { x: 11, y: 10 },
            create::GamestateChunk {
                coord: coord::GamestateCoord { x: 11, y: 10 },
                terrain: vec![vec!['l', 'm'], vec!['L', 'M']],
                users: HashMap::new(),
            },
        ),
        (
            coord::GamestateCoord { x: 9, y: 11 },
            create::GamestateChunk {
                coord: coord::GamestateCoord { x: 9, y: 11 },
                terrain: vec![vec!['n', 'o'], vec!['N', 'O']],
                users: HashMap::new(),
            },
        ),
        (
            coord::GamestateCoord { x: 10, y: 11 },
            create::GamestateChunk {
                coord: coord::GamestateCoord { x: 10, y: 11 },
                terrain: vec![vec!['p', 'q'], vec!['P', 'Q']],
                users: HashMap::new(),
            },
        ),
        (
            coord::GamestateCoord { x: 11, y: 11 },
            create::GamestateChunk {
                coord: coord::GamestateCoord { x: 11, y: 11 },
                terrain: vec![vec!['r', 's'], vec!['R', 'S']],
                users: HashMap::new(),
            },
        ),
    ]);

    let visible_gamestate =
        create_visible_gamestate(coord::GamestateCoord { x: 10, y: 10 }, chunks);

    assert_eq!(
        visible_gamestate,
        VisibleGamestate {
            terrain: vec![
                vec!['a', 'b', 'c', 'd', 'e', 'f'],
                vec!['A', 'B', 'C', 'D', 'E', 'F'],
                vec!['h', 'i', 'j', 'k', 'l', 'm'],
                vec!['H', 'I', 'J', 'K', 'L', 'M'],
                vec!['n', 'o', 'p', 'q', 'r', 's'],
                vec!['N', 'O', 'P', 'Q', 'R', 'S'],
            ],
            users: HashMap::new()
        }
    )
}
