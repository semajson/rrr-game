use regex::{Match, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    DELETE,
    OPTIONS,
}
impl HttpMethod {
    fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::DELETE => "DELETE",
        }
    }
    fn new(raw: &str) -> HttpMethod {
        match raw {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "OPTIONS" => HttpMethod::OPTIONS,
            "DELETE" => HttpMethod::DELETE,
            _ => panic!(
                "Received invalid method {} - this should be impossible",
                raw
            ),
        }
    }
}

#[derive(Debug)]
pub enum HttpErrorCode {
    Error400BadRequest,
    Error401Unauthorized,
    Error403Forbidden,
    Error404NotFround,
    Error409Conflict,
    Error500InternalServerError,
    Error501NotImplemented,
    Error503ServiceUnavailable,
}

pub fn build_response(response_body: Result<Response, HttpError>) -> String {
    let response = match response_body {
        Ok(response) => response,
        Err(error) => {
            let error_body = error_body(error.message);
            let error_status = match error.code {
                HttpErrorCode::Error400BadRequest => "400 Bad request".to_string(),
                HttpErrorCode::Error401Unauthorized => "401 Unauthorized".to_string(),
                HttpErrorCode::Error403Forbidden => "403 Forbidden".to_string(),
                HttpErrorCode::Error404NotFround => "404 Not found".to_string(),
                HttpErrorCode::Error409Conflict => "409 Conflict".to_string(),
                HttpErrorCode::Error501NotImplemented => "501 Not implemented".to_string(),
                HttpErrorCode::Error503ServiceUnavailable => "503 Service unavailable".to_string(),
                HttpErrorCode::Error500InternalServerError => {
                    "500 Internal Server Error".to_string()
                }
            };
            Response {
                body: error_body,
                status: error_status,
                headers: HashMap::new(),
            }
        }
    };

    let length = response.body.len();
    let response_status = response.status;
    let response_body = response.body;

    let mut response_raw = format!("HTTP/1.1 {response_status}\r\n");

    for (header, value) in &response.headers {
        response_raw += &format!("{header}: {value}\r\n");
    }

    response_raw += &format!("Content-Length: {length}\r\n\r\n{response_body}");

    println!("{:?}\n", response_raw); // Todo - logging
    response_raw
}

pub fn build_options_response_headers(allowed_methods: Vec<HttpMethod>) -> HashMap<String, String> {
    let allowed_methods = allowed_methods
        .iter()
        .map(|method| method.as_str())
        .collect::<Vec<&str>>();
    let allowed_methods = allowed_methods.join(", ");

    let mut headers = HashMap::new();

    headers.insert("Connection".to_string(), "keep-alive".to_string());
    headers.insert(
        "Access-Control-Allow-Origin".to_string(),
        "http://localhost:5500".to_string(),
    );
    headers.insert("Access-Control-Allow-Methods".to_string(), allowed_methods);
    headers.insert(
        "Access-Control-Allow-Headers".to_string(),
        "keep-alive, Content-Type".to_string(),
    );
    headers.insert("Access-Control-Max-Age".to_string(), "86400".to_string());

    headers
}

#[derive(Serialize, Deserialize)]
struct ErrorMsg {
    error_message: String,
}

fn error_body(error_message: String) -> String {
    let error_body = ErrorMsg { error_message };
    serde_json::to_string(&error_body).unwrap()
}

#[derive(Debug)]
pub struct HttpError {
    pub code: HttpErrorCode,
    pub message: String,
}

#[derive(Debug)]
pub struct Request {
    pub method: HttpMethod,
    pub resource: String,
    pub id: Option<String>,
    pub sub_resource: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub parameters: Option<Vec<(String, String)>>,
}

impl Request {
    pub fn new(request: String) -> Option<Request> {
        let request = request
            .lines()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        // Todo - maybe make this more efficient?
        // e.g. pares the request line first, and it it doesn't match, then bail
        let mut request_line = "".to_string();
        let mut headers = HashMap::new();
        let mut body = "".to_string();
        let mut in_body = false;
        for (index, line) in request.into_iter().enumerate() {
            if index == 0 {
                request_line = line;
            } else if line.is_empty() {
                in_body = true
            } else if in_body {
                body.push_str(&line);
            } else {
                // Todo = error parsing?
                let (header, value) = line.split_once(": ").unwrap();
                headers.insert(header.to_string(), value.to_string());
            }
        }

        // Parse request line
        let re = Regex::new(r"(?<method>GET|POST|DELETE|OPTIONS) (?<resource>/[a-z-]*)(?<id>/[a-zA-Z0-9]+)?(?<sub_resource>/[a-z0-9]+)?(?<parameters>\?[a-z-0-9=&]+)? HTTP/1.1").unwrap();
        let cap = re.captures_iter(&request_line).last();

        if let Some(valid_request) = cap {
            fn match_to_string_no_leading_char(option: Option<Match>) -> Option<String> {
                option.map(|value| value.as_str()[1..].to_string())
            }

            // Method and root must be present if the regex is matched
            let method = valid_request.name("method").unwrap().as_str().to_string();
            let method = HttpMethod::new(&method);

            let resource = valid_request.name("resource").unwrap().as_str().to_string();
            let id: Option<String> = match_to_string_no_leading_char(valid_request.name("id"));
            let sub_resource: Option<String> =
                match_to_string_no_leading_char(valid_request.name("sub_resource"));
            let parameters: Option<String> =
                match_to_string_no_leading_char(valid_request.name("parameters"));
            let parameters = match parameters {
                Some(raw) => {
                    let params = raw.split('&').map(String::from).collect::<Vec<String>>();

                    let params = params
                        .into_iter()
                        .map(|x| x.split('=').map(String::from).collect::<Vec<String>>())
                        .filter(|param| param.len() == 2) // This means invalid params are ignored
                        .map(|param| (param[0].clone(), param[1].clone()))
                        .collect();

                    Some(params)
                }
                None => None,
            };

            let request = Request {
                method,
                resource,
                id,
                sub_resource,
                headers,
                body,
                parameters,
            };

            Some(request)
        } else {
            None
        }
    }
}

pub struct Response {
    pub status: String,
    pub body: String,
    pub headers: HashMap<String, String>,
}
impl Response {
    pub fn response_from_body(body: Result<String, HttpError>) -> Result<Response, HttpError> {
        Ok(Response {
            body: body?,
            headers: HashMap::new(),
            status: "200 OK".to_string(),
        })
    }
}
