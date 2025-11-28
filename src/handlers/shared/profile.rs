use std::collections::HashMap;

use actix_web::{HttpRequest, patch, post, web};
use chrono::NaiveDateTime;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{handlers::auth::phone_verification::SuccessResponse, utils::{
    api_response::ApiResponse, http_client::ApiClient, jwt::get_logged_in_user_claims, validator_error::ValidationError,
}};

#[derive(Serialize, Deserialize, Debug)]
pub struct RolesData {
    #[serde(default)]
    pid: Uuid,
    #[serde(default)]
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    #[serde(default)]
    pub pid: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub is_email_verified: bool,
    #[serde(default)]
    pub country_code: String,
    #[serde(default)]
    pub phone_number: String,
    #[serde(default)]
    pub is_phone_verified: bool,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub last_login: NaiveDateTime,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub is_secret_verified: bool,
    #[serde(default)]
    pub tenant_name: Option<String>,
    #[serde(default)]
    pub roles: Vec<RolesData>,
    #[serde(default)]
    pub created_at: NaiveDateTime,
    pub message: String,
}

pub async fn get_profile_data(req: &HttpRequest) -> Result<UserResponse, ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;

    let api = ApiClient::new();
    let endpoint = format!("users/show/{}", claims.sub);
    let profile: UserResponse = api
        .call(&endpoint, &req, None::<&()>, Method::GET)
        .await
        .map_err(|err| {
            log::error!("auth/login API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Login failed. Please try again."
                }),
            )
        })?;

    Ok(profile)
}

pub async fn get_user_role_ids(req: &HttpRequest) -> Result<Vec<Uuid>, ApiResponse> {
    let profile = get_profile_data(&req).await.map_err(|err| {
        ApiResponse::new(
            500,
            json!({
                "message": err.to_string()
            }),
        )
    })?;

    let role_ids = profile
        .roles
        .into_iter()
        .map(|r| r.pid)
        .collect::<Vec<Uuid>>();

    Ok(role_ids)
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct TotpResponse {
    pub otpauth_url: Option<String>,
    pub secret: Option<String>,
    pub message: String,
}

#[post("/totp")]
async fn enable_2fa_totp(
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();
    
    let totp: TotpResponse = api
        .call("users/me/totp", &req, None::<&()>, Method::POST)
        .await
        .map_err(|err| {
            log::error!("users/me/totp API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Enable TOTP failed. Please try again."
                }),
            )
        })?;

    if totp.otpauth_url.is_none() {
        return Ok(ApiResponse::new(400, json!({
            "message": totp.message
        })))
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "otpauth_url": totp.otpauth_url,
            "secret": totp.secret,
            "message": totp.message
        }),
    ))
}

#[derive(Serialize, Deserialize, Debug)]
struct VerifyTotp {
    code: String,
}

impl VerifyTotp {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.code.trim().is_empty() {
            errors.insert("code".to_string(), "TOTP code is required".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[patch("/totp")]
async fn edit_2fa_totp(
    data: web::Json<VerifyTotp>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let api = ApiClient::new();
    
    let totp: TotpResponse = api
        .call("users/me/totp", &req, Some(&data), Method::PATCH)
        .await
        .map_err(|err| {
            log::error!("users/me/totp API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Edit TOTP failed. Please try again."
                }),
            )
        })?;

    if totp.otpauth_url.is_none() {
        return Ok(ApiResponse::new(400, json!({
            "message": totp.message
        })))
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "otpauth_url": totp.otpauth_url,
            "secret": totp.secret,
            "message": totp.message
        }),
    ))
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct VerifyTotpResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_codes: Option<Vec<String>>,
    pub message: String,
}

#[post("/verify_totp")]
async fn verify_totp_data(
    req: HttpRequest,
    data: web::Json<VerifyTotp>,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let api = ApiClient::new();
    
    let totp: VerifyTotpResponse = api
        .call("users/me/verify_totp", &req, Some(&data), Method::POST)
        .await
        .map_err(|err| {
            log::error!("users/me/verify_totp API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Verify TOTP failed. Please try again."
                }),
            )
        })?;

    if totp.recovery_codes.is_none() {
        return Ok(ApiResponse::new(400, json!({
            "message": totp.message
        })))
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "recovery_codes": totp.recovery_codes,
            "message": totp.message
        }),
    ))
}

#[derive(Serialize, Deserialize, Debug)]
struct VerifyRecoveryCodes {
    password: String,
}

impl VerifyRecoveryCodes {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.password.trim().is_empty() {
            errors.insert("password".to_string(), "Password is required".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[post("/regenerate_codes")]
async fn regenerate_recovery_codes(
    req: HttpRequest,
    data: web::Json<VerifyRecoveryCodes>,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let api = ApiClient::new();
    
    let totp: VerifyTotpResponse = api
        .call("users/me/regenerate_codes", &req, Some(&data), Method::POST)
        .await
        .map_err(|err| {
            log::error!("users/me/regenerate_codes API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Regenerate recovery codes failed. Please try again."
                }),
            )
        })?;

    if totp.recovery_codes.is_none() {
        return Ok(ApiResponse::new(400, json!({
            "message": totp.message
        })))
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "recovery_codes": totp.recovery_codes,
            "message": totp.message
        }),
    ))
}

#[post("/enable_otp")]
async fn enable_otp(
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
        let api = ApiClient::new();
    
    let otp: SuccessResponse = api
        .call("users/me/enable_otp", &req, None::<&()>, Method::POST)
        .await
        .map_err(|err| {
            log::error!("users/me/enable_otp API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Enable OTP failed. Please try again."
                }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": otp.message
        }),
    ))
}