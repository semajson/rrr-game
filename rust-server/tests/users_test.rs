use rust_book_server_example::{process_request, Database, LocalDatabase};
use std::sync::Arc;

mod util;

#[test]
fn test_create_user() {
    // Setup
    let db = Arc::new(LocalDatabase::new());
    let (user1, user2) = util::test_users();

    // Create user
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

    // Verify create user
    assert_eq!(response.status_code, 200);
    assert_ne!(db.get(&user1.username), None);

    // Invalid create user request - bad body
    let request = util::build_request(
        "POST",
        "/users",
        &format!(
            "{{\"username_blah_blah\":\"{}\", \"email\":\"{}\", \"password\":\"{}\"}}",
            user2.username, user2.email, user2.password
        ),
        "",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify failed
    assert_eq!(response.status_code, 400);
    assert_eq!(db.get(&user2.username), None);
}

#[test]
fn test_login() {
    // Setup
    let db = Arc::new(LocalDatabase::new());
    let (user1, _) = util::test_users();
    let request = util::build_request(
        "POST",
        "/users",
        &format!(
            "{{\"username\":\"{}\", \"email\":\"{}\", \"password\":\"{}\"}}",
            user1.username, user1.email, user1.password
        ),
        "",
    );
    process_request(request, Arc::clone(&db));

    // Valid login
    let request = util::build_request(
        "POST",
        "/sessions",
        &format!(
            "{{\"username\":\"{}\", \"password\":\"{}\"}}",
            user1.username, user1.password
        ),
        "",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify succeeded
    assert_eq!(response.status_code, 200);

    // Login request using wrong password
    let request = util::build_request(
        "POST",
        "/sessions",
        &format!(
            "{{\"username\":\"{}\", \"password\":\"{}\"}}",
            user1.username,
            "wrong_password".to_string()
        ),
        "",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify login failed
    assert_eq!(response.status_code, 401);

    // Invalid login request
    let request = util::build_request(
        "POST",
        "/sessions",
        &format!(
            "{{\"username\":\"{}\", \"password_abc\":\"{}\"}}",
            user1.username,
            "wrong_password".to_string()
        ),
        "",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify failed
    assert_eq!(response.status_code, 400);
}

#[test]
fn test_token() {
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
    process_request(request, Arc::clone(&db));

    // Get token from valid login
    let request = util::build_request(
        "POST",
        "/sessions",
        &format!(
            "{{\"username\":\"{}\", \"password\":\"{}\"}}",
            user1.username, user1.password
        ),
        "",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);
    let token = response.token.unwrap();

    // Verfiy token can be used
    let request = util::build_request("GET", &format!("/users/{}", user1.username), "", &token);
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);
    assert_eq!(response.status_code, 200);

    // Get token from create user
    let request = util::build_request(
        "POST",
        "/users",
        &format!(
            "{{\"username\":\"{}\", \"email\":\"{}\", \"password\":\"{}\"}}",
            user2.username, user2.email, user2.password
        ),
        "",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);
    let token = response.token.unwrap();

    // Verfiy token can be used
    let request = util::build_request("GET", &format!("/users/{}", user2.username), "", &token);
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);
    assert_eq!(response.status_code, 200);

    // Try to use a bad token
    let request = util::build_request(
        "GET",
        &format!("/users/{}", user2.username),
        "",
        "blah blah bad token",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify get 401
    assert_eq!(response.status_code, 401);

    // TODO: Try to access a protected resoruce without the auth header
    // let request = util::build_request("GET", &format!("/users/{}", user2.username), "", "");
    // let response = process_request(request, Arc::clone(&db));
    // let response = util::parse_response(response);

    // // Verify get 401
    // assert_eq!(response.status_code, 401);

    // Try to access protected resource with blank token
    let request = util::build_request("GET", &format!("/users/{}", user2.username), "", "");
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify get 401
    assert_eq!(response.status_code, 401);

    // Get valid token for user 1
    let request = util::build_request(
        "POST",
        "/sessions",
        &format!(
            "{{\"username\":\"{}\", \"password\":\"{}\"}}",
            user1.username, user1.password
        ),
        "",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);
    let token = response.token.unwrap();

    // Try to use it for user 2
    let request = util::build_request("GET", &format!("/users/{}", user2.username), "", &token);
    let response = process_request(request, Arc::clone(&db));
    let response = util::parse_response(response);

    // Verify get 403
    assert_eq!(response.status_code, 403);
}

#[test]
fn test_users_options() {
    // Given
    let db = Arc::new(LocalDatabase::new());
    let request = "OPTIONS /users HTTP/1.1\r
Host: localhost:7878\r
Connection: keep-alive\r
Accept: */*\r
Access-Control-Request-Method: POST\r
Access-Control-Request-Headers: content-type\r
Origin: null\r
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36\r
Sec-Fetch-Mode: cors\r
Sec-Fetch-Site: cross-site\r
Sec-Fetch-Dest: empty\r
Accept-Encoding: gzip, deflate, br, zstd\r
Accept-Language: en-GB,en-US;q=0.9,en;q=0.8\r
\r
".to_string();

    // When
    let response = process_request(request, Arc::clone(&db));

    // Then
    assert!(response.contains("Access-Control-Allow-Origin"));
    assert!(response.contains("Access-Control-Allow-Headers"));
    assert!(response.contains("Access-Control-Max-Age"));
    let response = util::parse_response(response);
    assert_eq!(response.status_code, 200);
}

#[test]
fn test_cors_headers() {
    // Given
    let db = Arc::new(LocalDatabase::new());
    let (user1, user2) = util::test_users();

    // When
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

    // Then
    assert!(response.contains("Access-Control-Allow-Origin"));
    assert!(response.contains("Access-Control-Allow-Headers"));
    assert!(response.contains("Access-Control-Max-Age"));
}

// TODO: tests
// get user info
// update user info
// delete user
