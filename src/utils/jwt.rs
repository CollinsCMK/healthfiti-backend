use actix_web::{HttpMessage, HttpRequest};
use jsonwebtoken::{dangerous::insecure_decode, errors::Error};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::utils::api_response::ApiResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub tenant_pid: Option<Uuid>,
    pub role_pid: Uuid,
    pub role_name: String,
    pub application_pid: Option<Uuid>,
    pub device_id: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    pub iss: String,
    pub token_type: String,
}

pub fn decode_token(token: &str) -> Result<Claims, Error> {
    let token_data = insecure_decode::<Claims>(token)?;
    Ok(token_data.claims)
}

pub fn get_logged_in_user_claims(req: &HttpRequest) -> Result<Claims, ApiResponse> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| ApiResponse::new(401, json!({ "message": "Unauthorized" })))
}
