use std::collections::HashMap;

use actix_web::{HttpRequest, post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::utils::{
    api_response::ApiResponse, http_client::ApiClient, validator_error::ValidationError,
};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct OtpCode {
    #[serde(default)]
    pub otp_code: String,
    #[serde(default)]
    pub user_id: Uuid,
    #[serde(default)]
    pub method: String,
}

impl OtpCode {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

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

        if self.user_id.is_nil() {
            errors.insert("user_id".to_string(), "User Id is required".into());
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

#[derive(Serialize, Deserialize, Debug)]
struct LoginVerifyData {
    #[serde(default)]
    otp: OtpCode,
    #[serde(default)]
    domain: Option<String>,
}

impl LoginVerifyData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if let Err(mut e) = self.otp.validate() {
            errors.extend(e.errors.drain());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginSuccessData {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub role_permissions: Value,
    pub expires_in_seconds: i64,
}

#[derive(Deserialize, Debug)]
pub struct LoginVerifyResponse {
    #[serde(default)]
    pub data: Option<LoginSuccessData>,
    pub message: String,
}

#[post("/login_verify")]
pub async fn login_verify(
    data: web::Json<LoginVerifyData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let api = ApiClient::new();

    let verify_response: LoginVerifyResponse = api
        .call(
            "auth/login_verify",
            &Some(req.clone()),
            Some(&*data),
            Method::POST,
        )
        .await
        .map_err(|err| {
            log::error!("Login verify external API error: {}", err);
            use std::error::Error;

            if let Some(source) = err.source() {
                log::error!("Cause: {:?}", source);
            }
            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to verify login. Please try again."
                }),
            )
        })?;

    if verify_response.data.is_none() {
        return Err(ApiResponse::new(
            500,
            json!({
                "message": verify_response.message
            }),
        ));
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "data": verify_response.data,
            "message": verify_response.message
        }),
    ))
}
