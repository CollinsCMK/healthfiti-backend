use std::collections::HashMap;

use actix_web::{HttpRequest, post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::utils::{
    api_response::ApiResponse, http_client::ApiClient, validator_error::ValidationError,
};

#[derive(Debug, Serialize, Deserialize)]
struct VerifyEmailData {
    #[serde(default)]
    user_id: Uuid,
    #[serde(default)]
    otp_code: String,
}

impl VerifyEmailData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.user_id.is_nil() {
            errors.insert(
                "user_id".to_string(),
                "User public Id is required".to_string(),
            );
        }

        if self.otp_code.is_empty() {
            errors.insert("otp_code".to_string(), "OTP Code is required".to_string());
        }

        if self.otp_code.len() != 6 || !self.otp_code.chars().all(|c| c.is_digit(10)) {
            errors.insert(
                "otp_code".to_string(),
                "OTP code must be a 6-digit numeric value.".into(),
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
struct EmailResponse {
    #[serde(default)]
    is_verify_phone: Option<bool>,
    message: String,
}

#[post("/verify_email")]
pub async fn verify_email(
    data: web::Json<VerifyEmailData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let api = ApiClient::new();

    let verify_email: EmailResponse = api
        .call("auth/verify_email", &req, Some(&*data), Method::POST)
        .await
        .map_err(|err| {
            log::error!("Verify email API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to verify email. Please try again."
                }),
            )
        })?;

    if verify_email.is_verify_phone.unwrap_or(false) {
        return Ok(ApiResponse::new(
            200,
            json!({
                "is_verify_phone": true,
                "message": verify_email.message
            }),
        ));
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": verify_email.message
        }),
    ))
}
