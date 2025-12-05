use std::collections::HashMap;

use actix_web::{HttpRequest, post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{handlers::services::tenants::ApiResponseDTO, utils::{
    api_response::ApiResponse, http_client::ApiClient, validator_error::ValidationError,
}};

#[derive(Debug, Serialize, Deserialize)]
struct VerifyPhoneData {
    #[serde(default)]
    user_id: Uuid,
    #[serde(default)]
    otp_code: String,
}

impl VerifyPhoneData {
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
                "OTP code must be a 6-digit numeric value.".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[post("/verify_phone")]
pub async fn verify_phone(
    data: web::Json<VerifyPhoneData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let api = ApiClient::new();

    let verify_phone: ApiResponseDTO<()> = api
        .call(
            "auth/verify_phone",
            &Some(req.clone()),
            Some(&*data),
            Method::POST,
        )
        .await
        .map_err(|err| {
            log::error!("Verify phone number API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to verify phone number. Please try again."
                }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": verify_phone.message
        }),
    ))
}
