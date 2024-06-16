use regex::Regex;
use rust_book_server_example::{process_request, Database, LocalDatabase};
use std::sync::Arc;

mod util;

fn get_game_id(body: &String) -> String {
    println!("");
    let re = Regex::new("game_id\":\"(?<game_id>[a-zA-Z0-9]{7})\"").unwrap();

    assert_eq!(re.captures_iter(&body).count(), 1);

    let captures = re.captures_iter(&body);
    let capture = captures.last().unwrap();

    capture.name("game_id").unwrap().as_str().to_string()
}

#[test]
fn test_create_rrr_game() {
    // Setup
    let db = Arc::new(LocalDatabase::new());
    let (user1, _user2) = util::test_users();
    let request = util::build_request(
        "POST",
        "/users",
        &format!(
            "{{\"username\":\"{}\", \"email\":\"{}\", \"password\":\"{}\"}}",
            user1.username, user1.email, user1.password
        ),
        "",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);
    let token = response.token.unwrap();

    // Create game
    let request = util::build_request("POST", "/rrr-game", "", &token);
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify created successfully
    assert_eq!(response.status_code, 200);
    assert!(response.body.is_some());
    assert_eq!(get_game_id(&response.body.clone().unwrap()).len(), 7);
    assert!(response.body.clone().unwrap().contains("\"terrain\":[["));
    assert!(response.body.clone().unwrap().contains("\"users\":{"));
    assert!(response.body.clone().unwrap().contains("\"G\",\"G\","));
}

#[test]
fn test_get_gamestate() {
    // Setup
    let db = Arc::new(LocalDatabase::new());
    let (user1, _user2) = util::test_users();
    let request = util::build_request(
        "POST",
        "/users",
        &format!(
            "{{\"username\":\"{}\", \"email\":\"{}\", \"password\":\"{}\"}}",
            user1.username, user1.email, user1.password
        ),
        "",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);
    let token = response.token.unwrap();

    // Create game
    let request = util::build_request("POST", "/rrr-game", "", &token);
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);
    assert_eq!(response.status_code, 200);
    assert!(response.body.is_some());
    let game_id = get_game_id(&response.body.clone().unwrap());

    // Get the gamestate
    let request = util::build_request("GET", &format!("/rrr-game/{}?x=0&y=0", game_id), "", &token);
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify
    assert_eq!(response.status_code, 200);
    assert!(response.body.is_some());
    assert!(response.body.clone().unwrap().contains("\"terrain\":[["));
    assert!(response.body.clone().unwrap().contains("\"users\":{"));
    assert!(response.body.clone().unwrap().contains("\"G\",\"G\","));
    assert_eq!(response.status_code, 200);
}

#[test]
fn test_make_move() {
    // Setup
    let db = Arc::new(LocalDatabase::new());
    let (user1, _user2) = util::test_users();
    let request = util::build_request(
        "POST",
        "/users",
        &format!(
            "{{\"username\":\"{}\", \"email\":\"{}\", \"password\":\"{}\"}}",
            user1.username, user1.email, user1.password
        ),
        "",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);
    let token = response.token.unwrap();

    // Manually insert a gamestate chunk into the DB
    let gamestate_chunk = "{
            \"coord\":
                {
                    \"x\":0,
                    \"y\":0
                },
            \"terrain\":
                [[\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\"],
                [\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\"],
                [\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\"],
                [\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\"],
                [\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\"],
                [\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\"],
                [\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\"],
                [\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\"],
                [\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\",\"G\"]],
            \"users\":
                {
                    \"james\":
                        {
                            \"x\":0,
                            \"y\":0
                        }
                }
            }";
    db.set(
        "rrr-game:1234567:0-0".to_string(),
        gamestate_chunk.to_string(),
    );

    // Make move East
    let request = util::build_request(
        "POST",
        "/rrr-game/1234567/actions?x=0&y=0",
        &format!("{{\"move\":\"East\"}}"),
        &token,
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify
    assert!(response.body.is_some());
    assert!(response.body.clone().unwrap().contains("\"x\":1,\"y\":0"));
    assert_eq!(response.status_code, 200);

    // Make move East
    let request = util::build_request(
        "POST",
        "/rrr-game/1234567/actions?x=0&y=0",
        &format!("{{\"move\":\"East\"}}"),
        &token,
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify
    assert!(response.body.is_some());
    assert!(response.body.clone().unwrap().contains("\"x\":2,\"y\":0"));
    assert_eq!(response.status_code, 200);

    // Make move South
    let request = util::build_request(
        "POST",
        "/rrr-game/1234567/actions?x=0&y=0",
        &format!("{{\"move\":\"South\"}}"),
        &token,
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify
    assert!(response.body.is_some());
    assert!(response.body.clone().unwrap().contains("\"x\":2,\"y\":1"));
    assert_eq!(response.status_code, 200);

    // Make move West
    let request = util::build_request(
        "POST",
        "/rrr-game/1234567/actions?x=0&y=0",
        &format!("{{\"move\":\"West\"}}"),
        &token,
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify
    assert!(response.body.is_some());
    assert!(response.body.clone().unwrap().contains("\"x\":1,\"y\":1"));
    assert_eq!(response.status_code, 200);

    // Make move Noth
    let request = util::build_request(
        "POST",
        "/rrr-game/1234567/actions?x=0&y=0",
        &format!("{{\"move\":\"North\"}}"),
        &token,
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify
    assert!(response.body.is_some());
    assert!(response.body.clone().unwrap().contains("\"x\":1,\"y\":0"));
    assert_eq!(response.status_code, 200);
}
