use regex::{Match, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const GET: &str = "GET";
pub const POST: &str = "POST";
pub const DELETE: &str = "DELETE";

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

pub fn build_response(response_body: Result<String, HttpError>) -> String {
    let (rsp_status, rsp_body) = match response_body {
        Ok(body) => ("200 OK", body),
        Err(error) => {
            let error_body = error_body(error.message);
            match error.code {
                HttpErrorCode::Error400BadRequest => ("400 Bad request", error_body),
                HttpErrorCode::Error401Unauthorized => ("401 Unauthorized", error_body),
                HttpErrorCode::Error403Forbidden => ("403 Forbidden", error_body),
                HttpErrorCode::Error404NotFround => ("404 Not found", error_body),
                HttpErrorCode::Error409Conflict => ("409 Conflict", error_body),
                HttpErrorCode::Error501NotImplemented => ("501 Not implemented", error_body),
                HttpErrorCode::Error503ServiceUnavailable => {
                    ("503 Service unavailable", error_body)
                }
                HttpErrorCode::Error500InternalServerError => {
                    ("500 Internal Server Error", error_body)
                }
            }
        }
    };

    let length = rsp_body.len();
    let response = format!("HTTP/1.1 {rsp_status}\r\nContent-Length: {length}\r\n\r\n{rsp_body}");
    response
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
    pub method: String,
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
        let re = Regex::new(r"(?<method>GET|POST|DELETE) (?<resource>/[a-z-]*)(?<id>/[a-zA-Z0-9]+)?(?<sub_resource>/[a-z0-9]+)?(?<parameters>\?[a-z-0-9=&]+)? HTTP/1.1").unwrap();
        let cap = re.captures_iter(&request_line).last();

        if let Some(valid_request) = cap {
            fn match_to_string_no_leading_char(option: Option<Match>) -> Option<String> {
                option.map(|value| value.as_str()[1..].to_string())
            }

            // Method and root must be present if the regex is matched
            let method = valid_request.name("method").unwrap().as_str().to_string();
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
