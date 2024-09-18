use crate::{
    http::{HttpError, HttpErrorCode},
    rrr_game::{coord, get, CHUNK_LENGTH, GAME_NAME},
    users, Database,
};
use rand::{distributions::Alphanumeric, Rng}; // 0.8
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub const TILE_GRASS: char = 'G';
pub const TILE_ROCK: char = 'R';
pub const TILE_WATER: char = 'W';

#[derive(Serialize, Deserialize, Clone)]
pub struct GamestateChunk {
    pub coord: coord::GamestateCoord,
    pub terrain: Vec<Vec<char>>,
    pub users: HashMap<String, coord::UserCoord>,
}
impl GamestateChunk {
    pub fn get_id(&self) -> String {
        self.coord.id()
    }
    fn new(
        coord: coord::GamestateCoord,
        username: &str,
        user_coord: Option<coord::UserCoord>,
    ) -> GamestateChunk {
        let mut terrain = vec![vec![TILE_GRASS; CHUNK_LENGTH]; CHUNK_LENGTH];

        let mut rng = rand::thread_rng();
        for row in terrain.iter_mut() {
            for i in 0..row.len() {
                let y: f64 = rng.gen();

                if y < 0.1 {
                    row[i] = TILE_WATER;
                } else if y > 0.9 {
                    row[i] = TILE_ROCK;
                }
            }
        }

        let mut users = HashMap::new();

        if let Some(user_coord) = user_coord {
            users.insert(username.to_owned(), user_coord);
        }

        GamestateChunk {
            coord,
            terrain,
            users,
        }
    }

    pub fn get_neighbours(&self) -> Vec<coord::GamestateCoord> {
        let mut neighbours = vec![];
        for dx in -1..=1 {
            for dy in -1..=1 {
                if (dx == 0) && (dy == 0) {
                    continue;
                }
                neighbours.push(coord::GamestateCoord {
                    x: self.coord.x + dx,
                    y: self.coord.y + dy,
                });
            }
        }
        neighbours
    }
}

#[derive(Serialize, Deserialize)]
struct CreateGameRsp {
    game_id: String,
    user_coord: coord::UserCoord,
    top_left_visible_coord: coord::UserCoord,
    visible_gamestate: get::VisibleGamestate, // Todo - decide if want to do this or not
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
    let centre_chunk_coord = coord::GamestateCoord { x: 0, y: 0 };
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
    let user_coord = coord::UserCoord { x: 0, y: 0 };
    let centre_chunk = GamestateChunk::new(
        centre_chunk_coord.clone(),
        &username,
        Some(user_coord.clone()),
    );

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
    let visible_gamestate = get::get_visible_gamestate(&user_coord, username, &game_id, db)?;
    let top_left_visible_coord =
        coord::get_top_left_visible_coord(&centre_chunk_coord, CHUNK_LENGTH);
    let rsp = CreateGameRsp {
        game_id,
        user_coord,
        top_left_visible_coord,
        visible_gamestate,
    };
    Ok(serde_json::to_string(&rsp).unwrap())
}
