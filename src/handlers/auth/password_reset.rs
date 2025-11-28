use std::collections::HashMap;

use actix_web::{HttpRequest, post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    handlers::auth::{phone_verification::SuccessResponse, two_factor::OtpCode},
    utils::{
        api_response::ApiResponse, http_client::ApiClient, validation::validate_password,
        validator_error::ValidationError,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UpdatePasswordData {
    #[serde(default)]
    user_id: Uuid,
    #[serde(default)]
    otp_code: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    confirm_password: String,
    #[serde(default)]
    method: String,
    #[serde(default)]
    domain: Option<String>,
}

impl UpdatePasswordData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.user_id.is_nil() {
            errors.insert(
                "user_id".to_string(),
                "User public Id is required.".to_string(),
            );
        }

        let method = self.method.trim().to_lowercase();
        if self.otp_code.is_empty() {
            let msg = match method.as_str() {
                "otp" => "OTP is required",
                "totp" => "TOTP code is required",
                "recovery" => "Recovery code is required",
                _ => "OTP Code is required",
            };
            errors.insert("otp_code".to_string(), msg.to_string());
        }

        if !validate_password(&self.password) {
            errors.insert("password".to_string(), "Password must have at least 8 characters, include an uppercase letter, a lowercase letter, a number, and a special character.".into());
        }

        if self.password != self.confirm_password {
            errors.insert(
                "confirm_password".to_string(),
                "Passwords do not match.".into(),
            );
        }

        if method.is_empty() {
            errors.insert("method".to_string(), "Method is required".to_string());
        } else if !["otp", "totp", "recovery"].contains(&method.as_str()) {
            errors.insert(
                "method".to_string(),
                "Method must be one of: otp, totp, or recovery".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

impl From<UpdatePasswordData> for OtpCode {
    fn from(value: UpdatePasswordData) -> Self {
        OtpCode {
            otp_code: value.otp_code,
            user_id: value.user_id,
            method: value.method,
        }
    }
}

#[post("/reset_password")]
pub async fn reset_password(
    data: web::Json<UpdatePasswordData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let api = ApiClient::new();

    let reset: SuccessResponse = api
        .call("auth/reset_password", &req, Some(&*data), Method::POST)
        .await
        .map_err(|err| {
            log::error!("Reset password external API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to reset password. Please try again."
                }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": reset.message
        }),
    ))
}
