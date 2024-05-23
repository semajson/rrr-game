use crate::{
    requests::{HttpError, HttpErrorCode},
    users, Database,
};
use rand::{distributions::Alphanumeric, Rng}; // 0.8
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Serialize, Deserialize)]
struct Coord {
    x: i32,
    y: i32,
}
impl Coord {
    fn id(&self) -> String {
        self.x.to_string() + "-" + &self.y.to_string()
    }
}

const GAME_NAME: &str = "rrr-game";

const CHUNK_LENGTH: usize = 10;
const GRASS: char = 'G';

#[derive(Serialize, Deserialize)]
struct GamestateChunk {
    coord: Coord,
    terrain: Vec<Vec<char>>,
    users: HashMap<String, Coord>,
}
impl GamestateChunk {
    fn get_id(&self) -> String {
        self.coord.id()
    }
    fn new(coord: Coord, username: &String, user_coord: Option<Coord>) -> GamestateChunk {
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

    fn get_neighbours(&self) -> Vec<Coord> {
        let mut neighbours = vec![];
        for dx in -1..=1 {
            for dy in -1..=1 {
                if (dx == 0) && (dy == 0) {
                    continue;
                }
                neighbours.push(Coord {
                    x: self.coord.x + dx,
                    y: self.coord.y + dy,
                });
            }
        }
        neighbours
    }
}

fn get_visible_gamestate(centre_chunk: GamestateChunk, chunks: Vec<GamestateChunk>) {
    // Todo
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

    // Get new current game ID
    let game_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let centre_chunk_cord = Coord { x: 0, y: 0 };
    if db
        .get(&(GAME_NAME.to_string() + ":" + &game_id + ":" + &centre_chunk_cord.id()))
        .is_some()
    {
        return Err(HttpError {
            code: HttpErrorCode::Error503ServiceUnavailable,
            message: "Clash when creating new game ID".to_string(),
        });
    }

    // Create new chunks
    let centre_chunk =
        GamestateChunk::new(centre_chunk_cord, &username, Some(Coord { x: 0, y: 0 }));

    let neighbours = centre_chunk.get_neighbours();
    let mut chunks = vec![centre_chunk];
    for neighbour in neighbours {
        chunks.push(GamestateChunk::new(neighbour, &username, None))
    }

    // Store chunks in DB
    for chunk in chunks {
        db.set(
            GAME_NAME.to_string() + ":" + &game_id + ":" + &chunk.get_id(),
            serde_json::to_string(&chunk).unwrap(),
        );
    }

    // todo - temp return the gamestate to the user

    Ok(serde_json::to_string(&centre_chunk).unwrap())
}
