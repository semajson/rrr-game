use regex::Regex;
use rust_book_server_example::{process_request, Database, LocalDatabase};
use std::str;
use std::sync::Arc;

mod util;

#[test]
fn test_create_rrr_game() {
    // Setup
    let db = Arc::new(LocalDatabase::new());
    let (user1, user2) = util::test_users();
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
    assert!(response.body.clone().unwrap().contains("\"terrain\":[["));
    assert!(response.body.clone().unwrap().contains("\"users\":{"));
    assert!(response.body.clone().unwrap().contains("\"G\",\"G\","));
}
