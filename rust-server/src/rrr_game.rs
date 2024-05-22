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
        self.coord.x.to_string() + "-" + &self.coord.y.to_string()
    }
    fn new(coord: Coord, username: &String, user_coord: Coord) -> GamestateChunk {
        let terrain = vec![vec![GRASS; CHUNK_LENGTH]; CHUNK_LENGTH];
        let users = HashMap::from([(username.clone(), user_coord)]);

        GamestateChunk {
            coord,
            terrain,
            users,
        }
    }
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
    if db
        .get(&(GAME_NAME.to_string() + &game_id + "-0-0"))
        .is_some()
    {
        return Err(HttpError {
            code: HttpErrorCode::Error503ServiceUnavailable,
            message: "Clash when creating new game ID".to_string(),
        });
    }

    // Create new chunks
    let new_chunk = GamestateChunk::new(Coord { x: 0, y: 0 }, &username, Coord { x: 0, y: 0 });

    // todo Create surrounding chunks

    // todo Store chunks in DB

    // TODO - temp return the gamestate to the user

    Ok(serde_json::to_string(&new_chunk).unwrap())
}
