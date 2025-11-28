use std::collections::HashMap;

use actix_web::{HttpRequest, post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils::{
    api_response::ApiResponse,
    http_client::ApiClient,
    validation::{validate_password, validate_phone_number},
    validator_error::ValidationError,
};

#[derive(Serialize, Deserialize, Debug)]
struct RegisterData {
    #[serde(default)]
    first_name: String,
    #[serde(default)]
    last_name: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    country_code: String,
    #[serde(default)]
    phone_number: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    confirm_password: String,
    #[serde(default)]
    domain: Option<String>,
}

impl RegisterData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.first_name.trim().is_empty() {
            errors.insert(
                "first_name".to_string(),
                "First Name is required".to_string(),
            );
        }

        if self.last_name.trim().is_empty() {
            errors.insert("last_name".to_string(), "Last Name is required".to_string());
        }

        if self.username.trim().is_empty() {
            errors.insert("username".to_string(), "Username is required".to_string());
        }

        if self.email.trim().is_empty() {
            errors.insert("email".into(), "Email address is required.".into());
        } else if !self.email.contains('@') {
            errors.insert(
                "email".into(),
                "Please provide a valid email address.".into(),
            );
        }

        if self.country_code.trim().is_empty() {
            errors.insert("country_code".into(), "Country code is required.".into());
        }

        if self.phone_number.trim().is_empty() {
            errors.insert(
                "phone_number".to_string(),
                "Phone number is required".to_string(),
            );
        }

        if !validate_phone_number(&self.phone_number) {
            errors.insert(
                "phone_number".to_string(),
                "Invalid phone number format".to_string(),
            );
        }

        if !validate_password(&self.password) {
            errors.insert("password".to_string(), "Password must be at least 8 characters long, contain at least one uppercase letter, one lowercase letter, one digit, and one special character (@$!%*?&)".to_string());
        }

        if self.confirm_password != self.password {
            errors.insert(
                "confirm_password".to_string(),
                "Confirm password must match the password".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[derive(Deserialize, Debug)]
struct RegisterResponse {
    #[serde(default)]
    is_verify_email: Option<bool>,
    #[serde(default)]
    user_id: Option<String>,
    #[serde(default)]
    tenant_pid: Option<String>,
    message: String,
}

#[post("/register")]
pub async fn register(
    data: web::Json<RegisterData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let api = ApiClient::new();

    let register: RegisterResponse = api
        .call("auth/register", &req, Some(&*data), Method::POST)
        .await
        .map_err(|err| {
            log::error!("Register user API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to register user. Please try again."
                }),
            )
        })?;

    if register.is_verify_email.unwrap_or(false) {
        return Ok(ApiResponse::new(
            200,
            json!({
                "is_verify_email": true,
                "user_id": register.user_id.expect("User ID"),
                "tenant_pid": register.tenant_pid.expect("Tenant ID"),
                "message": register.message
            }),
        ));
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": register.message
        }),
    ))
}
