use jsonwebtoken::{dangerous::insecure_decode, errors::Error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
