use regex::{Match, Regex};
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

struct Response {
    status_code: u32,
    body: String,
}

#[test]
fn create_user() {
    let db = Arc::new(LocalDatabase::new());

    // Create user
    let request = build_request(
        "POST",
        "/users",
        "{\"username\":\"james\", \"email\":\"james@gmail.com\", \"password\":\"testpassword\"}",
    );
    let response = process_request(request, Arc::clone(&db));
    let response = parse_response(response);

    // Verify create user
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, "");
    assert!(db.get("test") == Some("expected".to_string()));

    // Invalid create user request

    // Verify failed
}

// login test
// Successful login
// println!("{:?}", response);

// Verify login succeeded

// Unsuccessful login

// Verify login failed

// todo
// get user info
// update user info
// delete user
// logout
