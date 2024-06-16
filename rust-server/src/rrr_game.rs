use crate::{
    requests::{HttpError, HttpErrorCode},
    users, Database,
};
use rand::{distributions::Alphanumeric, Rng}; // 0.8
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
struct GamestateCoord {
    x: i32,
    y: i32,
}
impl GamestateCoord {
    fn id(&self) -> String {
        self.x.to_string() + "-" + &self.y.to_string()
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
struct UserCoord {
    x: i32,
    y: i32,
}

const GAME_NAME: &str = "rrr-game";

const CHUNK_LENGTH: usize = 9;
const GRASS: char = 'G';

#[derive(Serialize, Deserialize, Clone)]
struct GamestateChunk {
    coord: GamestateCoord,
    terrain: Vec<Vec<char>>,
    users: HashMap<String, UserCoord>,
}
impl GamestateChunk {
    fn get_id(&self) -> String {
        self.coord.id()
    }
    fn new(
        coord: GamestateCoord,
        username: &String,
        user_coord: Option<UserCoord>,
    ) -> GamestateChunk {
        let terrain = vec![vec![GRASS; CHUNK_LENGTH]; CHUNK_LENGTH];
        let mut users = HashMap::new();

        if let Some(user_coord) = user_coord {
            users.insert(username.clone(), user_coord);
        }

        GamestateChunk {
            coord,
            terrain,
            users,
        }
    }

    fn get_neighbours(&self) -> Vec<GamestateCoord> {
        let mut neighbours = vec![];
        for dx in -1..=1 {
            for dy in -1..=1 {
                if (dx == 0) && (dy == 0) {
                    continue;
                }
                neighbours.push(GamestateCoord {
                    x: self.coord.x + dx,
                    y: self.coord.y + dy,
                });
            }
        }
        neighbours
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
struct VisibleGamestate {
    terrain: Vec<Vec<char>>,
    users: HashMap<String, UserCoord>,
}

#[derive(Serialize, Deserialize)]
struct CreateGameRsp {
    game_id: String,
    user_coord: UserCoord,
    visible_gamestate: VisibleGamestate, // Todo - decide if want to do this or not
}

fn create_visible_gamestate(
    centre: GamestateCoord,
    chunks: HashMap<GamestateCoord, GamestateChunk>,
) -> VisibleGamestate {
    // Get users
    let mut users: HashMap<String, UserCoord> = HashMap::new();
    for (_, chunk) in chunks.iter() {
        users.extend(chunk.users.clone());
    }

    // Userful inline functions (?)
    let get_chunk = |dx, dy| {
        chunks
            .get(&GamestateCoord {
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

fn generate_game_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

pub fn create_game(username: String, db: Arc<impl Database>) -> Result<String, HttpError> {
    // Check if user is in a game already
    let curr_game_id = users::get_user_curr_game_info(&username, Arc::clone(&db), "rrr-game")?;

    if curr_game_id.is_some() {
        return Err(HttpError {
            code: HttpErrorCode::Error400BadRequest,
            message: "User is already in a game".to_string(),
        });
    }

    // Create new current game ID
    let game_id: String = generate_game_id();
    let centre_chunk_coord = GamestateCoord { x: 0, y: 0 };
    if db
        .get(&(GAME_NAME.to_string() + ":" + &game_id + ":" + &centre_chunk_coord.id()))
        .is_some()
    {
        return Err(HttpError {
            code: HttpErrorCode::Error503ServiceUnavailable,
            message: "Clash when creating new game ID".to_string(),
        });
    }

    // Create new chunks
    let user_coord = UserCoord { x: 0, y: 0 };
    let centre_chunk = GamestateChunk::new(centre_chunk_coord, &username, Some(user_coord.clone()));

    let neighbours = centre_chunk.get_neighbours();
    let mut chunks = HashMap::from([(centre_chunk.coord.clone(), centre_chunk.clone())]);
    for neighbour in neighbours {
        chunks.insert(
            neighbour.clone(),
            GamestateChunk::new(neighbour, &username, None),
        );
    }

    // Store chunks in DB
    for (_, chunk) in chunks {
        db.set(
            GAME_NAME.to_string() + ":" + &game_id + ":" + &chunk.get_id(),
            serde_json::to_string(&chunk).unwrap(),
        );
    }

    // Todo - consider if should hit db here - maybe just to be sure it was written?
    let visible_gamestate = get_visible_gamestate(&user_coord, username, &game_id, db)?;
    let rsp = CreateGameRsp {
        game_id,
        user_coord,
        visible_gamestate,
    };
    Ok(serde_json::to_string(&rsp).unwrap())
}

fn user_coord_to_gamestate_coord(user_coord: &UserCoord, chunk_length: usize) -> GamestateCoord {
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

fn get_visible_gamestate(
    user_coord: &UserCoord,
    username: String,
    game_id: &String,
    db: Arc<impl Database>,
) -> Result<VisibleGamestate, HttpError> {
    let centre_gamestate_coord = user_coord_to_gamestate_coord(user_coord, CHUNK_LENGTH);

    // Get centre gamestatechunk
    let centre_gamestate_chunk = db
        .get(&(GAME_NAME.to_string() + ":" + game_id + ":" + &centre_gamestate_coord.id()))
        .unwrap(); // Todo error handling - goes for all unwrap() calls
    let centre_gamestate_chunk: GamestateChunk =
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
    // Convert the passed in params to Usercoord
    let parameters = parameters.ok_or(HttpError {
        code: HttpErrorCode::Error400BadRequest,
        message: "User coords not supplied in GET gamestate request.".to_string(),
    })?;
    let user_coord = get_usercoord_from_params(parameters)?;

    let visible_gamestate = get_visible_gamestate(&user_coord, username, &game_id, db)?;
    Ok(serde_json::to_string(&visible_gamestate).unwrap())
}

fn get_usercoord_from_params(parameters: Vec<(String, String)>) -> Result<UserCoord, HttpError> {
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
    user_coord: UserCoord,
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
    let mut user_coord = get_usercoord_from_params(parameters)?;
    let gamestate_coord = user_coord_to_gamestate_coord(&user_coord, CHUNK_LENGTH);

    // Get the gamestate chunk
    let gamestate_chunk = db
        .get(&(GAME_NAME.to_string() + ":" + &game_id + ":" + &gamestate_coord.id()))
        .unwrap(); // Todo error handling - goes for all unwrap() calls
    let mut gamestate_chunk: GamestateChunk = serde_json::from_str(&gamestate_chunk).unwrap();

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
        Move::North => UserCoord {
            x: user_coord.x,
            y: user_coord.y - 1,
        },
        Move::East => UserCoord {
            x: user_coord.x + 1,
            y: user_coord.y,
        },
        Move::South => UserCoord {
            x: user_coord.x,
            y: user_coord.y + 1,
        },
        Move::West => UserCoord {
            x: user_coord.x - 1,
            y: user_coord.y,
        },
    };
    gamestate_chunk
        .users
        .insert(username, new_user_coord.clone());

    // Todo - deal with going off edge of chunk

    // Write to DB
    db.set(
        GAME_NAME.to_string() + ":" + &game_id + ":" + &gamestate_chunk.get_id(),
        serde_json::to_string(&gamestate_chunk).unwrap(),
    );

    // Return
    // Todo - workout if want to return the gamestate here...
    let rsp = ActionRsp {
        user_coord: new_user_coord,
    };
    Ok(serde_json::to_string(&rsp).unwrap())
}

#[test]
fn test_create_visible_gamestate() {
    let chunks: HashMap<GamestateCoord, GamestateChunk> = HashMap::from([
        (
            GamestateCoord { x: 9, y: 9 },
            GamestateChunk {
                coord: GamestateCoord { x: 9, y: 9 },
                terrain: vec![vec!['a', 'b'], vec!['A', 'B']],
                users: HashMap::new(),
            },
        ),
        (
            GamestateCoord { x: 10, y: 9 },
            GamestateChunk {
                coord: GamestateCoord { x: 10, y: 9 },
                terrain: vec![vec!['c', 'd'], vec!['C', 'D']],
                users: HashMap::new(),
            },
        ),
        (
            GamestateCoord { x: 11, y: 9 },
            GamestateChunk {
                coord: GamestateCoord { x: 11, y: 9 },
                terrain: vec![vec!['e', 'f'], vec!['E', 'F']],
                users: HashMap::new(),
            },
        ),
        (
            GamestateCoord { x: 9, y: 10 },
            GamestateChunk {
                coord: GamestateCoord { x: 9, y: 19 },
                terrain: vec![vec!['h', 'i'], vec!['H', 'I']],
                users: HashMap::new(),
            },
        ),
        (
            GamestateCoord { x: 10, y: 10 },
            GamestateChunk {
                coord: GamestateCoord { x: 10, y: 10 },
                terrain: vec![vec!['j', 'k'], vec!['J', 'K']],
                users: HashMap::new(),
            },
        ),
        (
            GamestateCoord { x: 11, y: 10 },
            GamestateChunk {
                coord: GamestateCoord { x: 11, y: 10 },
                terrain: vec![vec!['l', 'm'], vec!['L', 'M']],
                users: HashMap::new(),
            },
        ),
        (
            GamestateCoord { x: 9, y: 11 },
            GamestateChunk {
                coord: GamestateCoord { x: 9, y: 11 },
                terrain: vec![vec!['n', 'o'], vec!['N', 'O']],
                users: HashMap::new(),
            },
        ),
        (
            GamestateCoord { x: 10, y: 11 },
            GamestateChunk {
                coord: GamestateCoord { x: 10, y: 11 },
                terrain: vec![vec!['p', 'q'], vec!['P', 'Q']],
                users: HashMap::new(),
            },
        ),
        (
            GamestateCoord { x: 11, y: 11 },
            GamestateChunk {
                coord: GamestateCoord { x: 11, y: 11 },
                terrain: vec![vec!['r', 's'], vec!['R', 'S']],
                users: HashMap::new(),
            },
        ),
    ]);

    let visible_gamestate = create_visible_gamestate(GamestateCoord { x: 10, y: 10 }, chunks);

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
