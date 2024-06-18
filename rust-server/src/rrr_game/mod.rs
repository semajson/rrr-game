// GAME_NAME:
const GAME_NAME: &str = "rrr-game";
const CHUNK_LENGTH: usize = 9; // Currently, this is half like a global variable

mod create;
pub use create::create_game;

mod coord;

mod get;
pub use get::get_gamestate;

mod action;
pub use action::do_action;
