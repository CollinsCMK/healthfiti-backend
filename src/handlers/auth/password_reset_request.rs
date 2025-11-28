use std::collections::HashMap;

use actix_web::{HttpRequest, post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils::{
    api_response::ApiResponse, http_client::ApiClient, validation::validate_phone_number,
    validator_error::ValidationError,
};

#[derive(Serialize, Deserialize, Debug)]
struct PasswordRequestData {
    #[serde(default)]
    credential: Option<String>,
    #[serde(default)]
    country_code: Option<String>,
    #[serde(default)]
    phone_number: Option<String>,
    #[serde(default)]
    domain: Option<String>,
}

impl PasswordRequestData {
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

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[derive(Deserialize, Debug)]
struct PasswordResetResponse {
    #[serde(default)]
    is_password_reset_verify: Option<bool>,
    #[serde(default)]
    is_totp_verified: Option<bool>,
    #[serde(default)]
    user_id: Option<String>,
    message: String,
}

async fn password_reset_request(
    data: &web::Json<PasswordRequestData>,
    user_type: &str,
    req: &HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    let api = ApiClient::new();

    let request_json = json!({
        "credential": data.credential,
        "country_code": data.country_code,
        "phone_number": data.phone_number,
        "domain": data.domain,
        "user_type": user_type,
    });

    let password_reset: PasswordResetResponse = api
        .call(
            "auth/password_reset_request",
            &req,
            Some(&request_json),
            Method::POST,
        )
        .await
        .map_err(|err| {
            log::error!("Password reset request API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to request password reset. Please try again."
                }),
            )
        })?;

    if password_reset.is_password_reset_verify.unwrap_or(false) {
        return Ok(ApiResponse::new(
            200,
            json!({
                "is_password_reset_verify": true,
                "user_id": password_reset.user_id.expect("User ID"),
                "message": password_reset.message
            }),
        ));
    }

    if password_reset.is_totp_verified.unwrap_or(false) {
        return Ok(ApiResponse::new(
            200,
            json!({
                "is_totp_verified": true,
                "user_id": password_reset.user_id.expect("User ID"),
                "message": password_reset.message
            }),
        ));
    }

    Ok(ApiResponse::new(
        400,
        json!({
            "message": password_reset.message
        }),
    ))
}

#[post("/admin/password_reset_request")]
async fn admin_password_reset_request(
    data: web::Json<PasswordRequestData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    match password_reset_request(&data, "admin", &req).await {
        Ok(response) => Ok(response),
        Err(err) => {
            log::error!("Admin password reset request failed: {:?}", err);

            Err(ApiResponse::new(
                500,
                json!({ "message": "Internal server error" }),
            ))
        }
    }
}

#[post("/tenant/password_reset_request")]
async fn tenant_password_reset_request(
    data: web::Json<PasswordRequestData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    match password_reset_request(&data, "tenant", &req).await {
        Ok(response) => Ok(response),
        Err(err) => {
            log::error!("Admin password reset request failed: {:?}", err);

            Err(ApiResponse::new(
                500,
                json!({ "message": "Internal server error" }),
            ))
        }
    }
}

#[post("/normal/password_reset_request")]
async fn normal_password_reset_request(
    data: web::Json<PasswordRequestData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    match password_reset_request(&data, "normal", &req).await {
        Ok(response) => Ok(response),
        Err(err) => {
            log::error!("Admin password reset request failed: {:?}", err);

            Err(ApiResponse::new(
                500,
                json!({ "message": "Internal server error" }),
            ))
        }
    }
}
