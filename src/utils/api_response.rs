use actix_web::{HttpResponse, Responder, ResponseError, body::BoxBody, http::StatusCode};
use serde_json::Value;
use std::fmt::Display;

#[derive(Debug)]
pub struct ApiResponse {
    pub status_code: u16,
    pub body: String,
    response_code: StatusCode,
}

impl ApiResponse {
    pub fn new(status_code: u16, json_value: Value) -> Self {
        let body = serde_json::to_string(&json_value).unwrap();
        ApiResponse {
            status_code,
            body,
            response_code: StatusCode::from_u16(status_code).unwrap(),
        }
    }
}

impl Responder for ApiResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        HttpResponse::build(self.response_code)
            .content_type("application/json") // This is the key fix!
            .body(self.body)
    }
}

impl Display for ApiResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error: {} \n Status Code: {}",
            self.body, self.status_code
        )
    }
}

impl ResponseError for ApiResponse {
    fn status_code(&self) -> StatusCode {
        self.response_code
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code())
            .content_type("application/json")
            .body(self.body.clone())
    }
}
