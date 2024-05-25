use crate::{
    requests::{HttpError, HttpErrorCode},
    users, Database,
};
use rand::{distributions::Alphanumeric, Rng}; // 0.8
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
// Hmm, should have differnt coords for users and game chunks?
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

#[derive(Serialize, Deserialize, Clone)]
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
fn get_visible_gamestate(
    centre: Coord,
    chunks: HashMap<Coord, GamestateChunk>,
) -> (Vec<Vec<char>>, HashMap<String, Coord>) {
    // Get users
    let mut users: HashMap<String, Coord> = HashMap::new();
    for (_, chunk) in chunks.iter() {
        users.extend(chunk.users.clone());
    }

    // Get terrain
    let get_chunk = |dx, dy| {
        chunks
            .get(&Coord {
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

    let mut terrain: Vec<Vec<char>> = vec![];

    // Top
    terrain.extend(get_new_rows(-1));

    // Middle
    terrain.extend(get_new_rows(0));

    // Bottom
    terrain.extend(get_new_rows(1));

    (terrain, users)
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
    let mut chunks = HashMap::from([(centre_chunk.coord.clone(), centre_chunk.clone())]); // maybe make a hashmap?
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

    // todo - temp return the gamestate to the user

    Ok(serde_json::to_string("").unwrap())
}

#[test]
fn test_get_visible_gamestate() {
    let chunks: HashMap<Coord, GamestateChunk> = HashMap::from([
        (
            Coord { x: 9, y: 9 },
            GamestateChunk {
                coord: Coord { x: 9, y: 9 },
                terrain: vec![vec!['a', 'b'], vec!['A', 'B']],
                users: HashMap::new(),
            },
        ),
        (
            Coord { x: 10, y: 9 },
            GamestateChunk {
                coord: Coord { x: 10, y: 9 },
                terrain: vec![vec!['c', 'd'], vec!['C', 'D']],
                users: HashMap::new(),
            },
        ),
        (
            Coord { x: 11, y: 9 },
            GamestateChunk {
                coord: Coord { x: 11, y: 9 },
                terrain: vec![vec!['e', 'f'], vec!['E', 'F']],
                users: HashMap::new(),
            },
        ),
        (
            Coord { x: 9, y: 10 },
            GamestateChunk {
                coord: Coord { x: 9, y: 19 },
                terrain: vec![vec!['h', 'i'], vec!['H', 'I']],
                users: HashMap::new(),
            },
        ),
        (
            Coord { x: 10, y: 10 },
            GamestateChunk {
                coord: Coord { x: 10, y: 10 },
                terrain: vec![vec!['j', 'k'], vec!['J', 'K']],
                users: HashMap::new(),
            },
        ),
        (
            Coord { x: 11, y: 10 },
            GamestateChunk {
                coord: Coord { x: 11, y: 10 },
                terrain: vec![vec!['l', 'm'], vec!['L', 'M']],
                users: HashMap::new(),
            },
        ),
        (
            Coord { x: 9, y: 11 },
            GamestateChunk {
                coord: Coord { x: 9, y: 11 },
                terrain: vec![vec!['n', 'o'], vec!['N', 'O']],
                users: HashMap::new(),
            },
        ),
        (
            Coord { x: 10, y: 11 },
            GamestateChunk {
                coord: Coord { x: 10, y: 11 },
                terrain: vec![vec!['p', 'q'], vec!['P', 'Q']],
                users: HashMap::new(),
            },
        ),
        (
            Coord { x: 11, y: 11 },
            GamestateChunk {
                coord: Coord { x: 11, y: 11 },
                terrain: vec![vec!['r', 's'], vec!['R', 'S']],
                users: HashMap::new(),
            },
        ),
    ]);

    let visible_gamestate = get_visible_gamestate(Coord { x: 10, y: 10 }, chunks);

    assert_eq!(
        visible_gamestate,
        (
            vec![
                vec!['a', 'b', 'c', 'd', 'e', 'f'],
                vec!['A', 'B', 'C', 'D', 'E', 'F'],
                vec!['h', 'i', 'j', 'k', 'l', 'm'],
                vec!['H', 'I', 'J', 'K', 'L', 'M'],
                vec!['n', 'o', 'p', 'q', 'r', 's'],
                vec!['N', 'O', 'P', 'Q', 'R', 'S'],
            ],
            HashMap::new()
        )
    )
}
