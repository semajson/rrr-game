use regex::Regex;
use std::str;

pub fn build_request(method: &str, url: &str, body: &str, token: &str) -> String {
    let body_length = body.len();

    let request = format!(
        "{method} {url} HTTP/1.1\r
User-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r
Host: www.tutorialspoint.com\r
Content-Type: application/json\r
Authorization: Bearer {token}\r
Accept: */*\r
Connection: Keep-Alive\r
Content-Length: {body_length}\r
\r
{body}"
    );

    request
}

pub struct Response {
    pub status_code: u32,
    pub body: Option<String>,
    pub token: Option<String>,
}

pub fn parse_response(response: String) -> Response {
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

    // Body
    let body = if let Some(body) = capture.name("body") {
        Some(body.as_str().to_string())
    } else {
        None
    };

    // Token
    let token = if let Some(ref body_present) = body {
        // { }
        let re = Regex::new(r#""access_token":"(?<token>.*)""#).unwrap();

        let captures = re.captures_iter(&body_present);

        captures
            .last()
            .map(|value| value.name("token").unwrap().as_str().to_string())
    } else {
        None
    };

    Response {
        status_code,
        body,
        token,
    }
}

pub struct User {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub fn test_users() -> (User, User) {
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
