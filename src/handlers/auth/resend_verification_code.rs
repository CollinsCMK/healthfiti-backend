use std::collections::HashMap;

use actix_web::{HttpRequest, post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    handlers::services::tenants::ApiResponseDTO,
    utils::{api_response::ApiResponse, http_client::ApiClient, validator_error::ValidationError},
};

#[derive(Serialize, Deserialize, Debug)]
struct ResendOtpData {
    #[serde(default)]
    user_id: Uuid,
    #[serde(default)]
    otp_purpose: String,
    #[serde(default)]
    domain: Option<String>,
}

impl ResendOtpData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.user_id.is_nil() {
            errors.insert("user_id".to_string(), "User public Id is required".into());
        }

        if self.otp_purpose.trim().is_empty() {
            errors.insert(
                "otp_purpose".to_string(),
                "OTP purpose type is required.".to_string(),
            );
        }

        let valid_purposes = [
            "login",
            "email_verification",
            "phone_verification",
            "password_reset",
        ];
        if !valid_purposes.contains(&self.otp_purpose.as_str()) {
            errors.insert(
                "otp_purpose".to_string(),
                format!("Invalid OTP purpose. Must be one of: {:?}.", valid_purposes),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[post("/resend_otp")]
pub async fn resend_otp(
    data: web::Json<ResendOtpData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let api = ApiClient::new();

    let resend_otp: ApiResponseDTO<()> = api
        .call(
            "auth/resend_otp",
            &Some(req.clone()),
            Some(&*data),
            Method::POST,
        )
        .await
        .map_err(|err| {
            log::error!("Resend verification code API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to resend verification code. Please try again."
                }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": resend_otp.message
        }),
    ))
}
