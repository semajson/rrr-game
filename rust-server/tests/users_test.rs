use regex::Regex;
use rust_book_server_example::{process_request, Database, LocalDatabase};
use std::str;
use std::sync::Arc;

fn build_request(method: &str, url: &str, body: &str) -> String {
    let body_length = body.len();

    let request = format!(
        "{method} {url} HTTP/1.1\r
User-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r
Host: www.tutorialspoint.com\r
Content-Type: application/json\r
Accept: */*\r
Connection: Keep-Alive\r
Content-Length: {body_length}\r
\r
{body}"
    );

    println!("{:?}", request);
    println!("");

    request
}

struct Response {
    status_code: u32,
    body: String,
}

fn parse_response(response: String) -> Response {
    println!("{:?}", response);
    println!("");
    let re = Regex::new(
        r"(?s)HTTP/1.1 (?<status_code>[0-9]+) (?<status_text>[a-zA-Z ]+).*\r\n\r\n(?<body>.*)",
    )
    .unwrap();

    assert_eq!(re.captures_iter(&response).count(), 1);

    let captures = re.captures_iter(&response);
    let capture = captures.last().unwrap();

    let status_code = capture
        .name("status_code")
        .unwrap()
        .as_str()
        .parse::<u32>()
        .unwrap();

    let body = if let Some(body) = capture.name("body") {
        body.as_str().to_string()
    } else {
        "".to_string()
    };

    Response { status_code, body }
}

pub struct User {
    username: String,
    email: String,
    password: String,
}

fn test_users() -> (User, User) {
    let user1: User = User {
        username: "james".to_string(),
        email: "james@gmail.com".to_string(),
        password: "testpassword".to_string(),
    };

    let user2: User = User {
        username: "alex".to_string(),
        email: "alex@hotmail.com".to_string(),
        password: "passwordtest".to_string(),
    };

    (user1, user2)
}

#[test]
fn test_create_user() {
    let db = Arc::new(LocalDatabase::new());
    let (user1, user2) = test_users();

    // Create user
    let request = build_request(
        "POST",
        "/users",
        &format!(
            "{{\"username\":\"{}\", \"email\":\"{}\", \"password\":\"{}\"}}",
            user1.username, user1.email, user1.password
        ),
    );
    let response = process_request(request, Arc::clone(&db));
    let response = parse_response(response);

    // Verify create user
    assert_eq!(response.status_code, 200);
    assert_ne!(db.get(&user1.username), None);

    // Invalid create user request - bad body
    let request = build_request(
        "POST",
        "/users",
        &format!(
            "{{\"username_blah_blah\":\"{}\", \"email\":\"{}\", \"password\":\"{}\"}}",
            user2.username, user2.email, user2.password
        ),
    );
    let response = process_request(request, Arc::clone(&db));
    let response = parse_response(response);

    // Verify failed
    assert_eq!(response.status_code, 400);
    assert_eq!(db.get(&user2.username), None);
}

#[test]
fn test_login() {
    let db = Arc::new(LocalDatabase::new());
    let (user1, _) = test_users();
    let request = build_request(
        "POST",
        "/users",
        &format!(
            "{{\"username\":\"{}\", \"email\":\"{}\", \"password\":\"{}\"}}",
            user1.username, user1.email, user1.password
        ),
    );
    process_request(request, Arc::clone(&db));

    // Valid login
    let request = build_request(
        "POST",
        "/sessions",
        &format!(
            "{{\"username\":\"{}\", \"password\":\"{}\"}}",
            user1.username, user1.password
        ),
    );
    let response = process_request(request, Arc::clone(&db));
    let response = parse_response(response);

    // Verify succeeded
    assert_eq!(response.status_code, 200);

    // Login request using wrong password
    let request = build_request(
        "POST",
        "/sessions",
        &format!(
            "{{\"username\":\"{}\", \"password\":\"{}\"}}",
            user1.username,
            "wrong_password".to_string()
        ),
    );
    let response = process_request(request, Arc::clone(&db));
    let response = parse_response(response);

    // Verify failed
    assert_eq!(response.status_code, 401);

    // Invalid login request
    let request = build_request(
        "POST",
        "/sessions",
        &format!(
            "{{\"username\":\"{}\", \"password_abc\":\"{}\"}}",
            user1.username,
            "wrong_password".to_string()
        ),
    );
    let response = process_request(request, Arc::clone(&db));
    let response = parse_response(response);

    // Verify failed
    assert_eq!(response.status_code, 400);
}

// Token test - verify token works correclty

// todo
// get user info
// update user info
// delete user
// logout
