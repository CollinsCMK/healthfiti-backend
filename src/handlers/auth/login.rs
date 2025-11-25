use std::collections::HashMap;

use actix_web::{post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils::{
    api_response::ApiResponse,
    http_client::{ApiClient, EndpointType},
    validation::validate_phone_number,
    validator_error::ValidationError,
};

#[derive(Serialize, Deserialize, Debug)]
struct LoginData {
    #[serde(default)]
    credential: Option<String>,
    #[serde(default)]
    country_code: Option<String>,
    #[serde(default)]
    phone_number: Option<String>,
    #[serde(default)]
    password: String,
    #[serde(default)]
    domain: Option<String>,
}

impl LoginData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        let credential_empty = self
            .credential
            .as_ref()
            .map_or(true, |c| c.trim().is_empty());
        let phone_missing = self
            .phone_number
            .as_ref()
            .map_or(true, |p| p.trim().is_empty());
        let country_missing = self
            .country_code
            .as_ref()
            .map_or(true, |c| c.trim().is_empty());

        if credential_empty && (phone_missing || country_missing) {
            errors.insert(
                "credential".to_string(),
                "Provide credential (email/username) or phone number".to_string(),
            );
        }

        if let Some(phone) = &self.phone_number {
            if country_missing {
                errors.insert(
                    "country_code".to_string(),
                    "Country code is required when phone number is provided".to_string(),
                );
            } else if !validate_phone_number(phone) {
                errors.insert(
                    "phone_number".to_string(),
                    "Invalid phone number format".to_string(),
                );
            }
        }

        if self.password.is_empty() {
            errors.insert("password".to_string(), "Password is required".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    #[serde(default)]
    is_verify_email: Option<bool>,
    #[serde(default)]
    is_verify_phone: Option<bool>,
    #[serde(default)]
    is_totp_verified: Option<bool>,
    #[serde(default)]
    is_login_verify: Option<bool>,
    #[serde(default)]
    user_id: Option<String>,
    message: String,
}

// async fn login(
//     data: web::Json<LoginData>
// ) -> Result<ApiResponse, ApiResponse> {

// }

#[post("/admin/login")]
async fn admin_login(data: web::Json<LoginData>) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    let api = ApiClient::new();

    let login: LoginResponse = api
        .call("auth/login", EndpointType::Auth, Some(&*data), Method::POST)
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    if login.is_verify_email.unwrap_or(false) {
        return Ok(ApiResponse::new(
            200,
            json!({
                "is_verify_email": true,
                "user_id": login.user_id.expect("User ID"),
                "message": login.message
            }),
        ));
    }

    if login.is_verify_phone.unwrap_or(false) {
        return Ok(ApiResponse::new(
            200,
            json!({
                    "is_verify_phone": true,
                    "user_id": login.user_id.expect("User ID"),
                "message": login.message
            }),
        ));
    }

    if login.is_totp_verified.unwrap_or(false) {
        return Ok(ApiResponse::new(
            200,
            json!({
                        "is_totp_verified": true,
                    "user_id": login.user_id.expect("User ID"),
                "message": login.message
            }),
        ));
    }

    if login.is_login_verify.unwrap_or(false) {
        return Ok(ApiResponse::new(
            200,
            json!({
                    "is_login_verify": true,
                    "user_id": login.user_id.expect("User ID"),
                "message": login.message
            }),
        ));
    }

    Ok(ApiResponse::new(
        400,
        json!({
            "message": login.message
        }),
    ))
}
