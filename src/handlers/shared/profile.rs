use std::collections::HashMap;

use actix_web::{HttpRequest, get, patch, post, web};
use chrono::NaiveDateTime;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{ColumnTrait, QueryFilter},
    }, handlers::services::tenants::ApiResponseDTO, utils::{
        api_response::ApiResponse, app_state::AppState, http_client::ApiClient,
        jwt::get_logged_in_user_claims, validator_error::ValidationError,
    }
};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(default)]
pub struct RolesData {
    pub pid: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(default)]
pub struct UserResponse {
    pub pid: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub is_email_verified: bool,
    pub country_code: String,
    pub phone_number: String,
    pub is_phone_verified: bool,
    pub username: String,
    pub last_login: Option<NaiveDateTime>,
    pub is_enabled: Option<bool>,
    pub method: Option<String>,
    pub is_secret_verified: Option<bool>,
    pub tenant_name: Option<String>,
    pub roles: Vec<RolesData>,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct ProfileDTO {
    pub user: Option<UserResponse>,
    pub message: String,
}

pub async fn get_profile_data(req: &HttpRequest) -> Result<ProfileDTO, ApiResponse> {
    let api = ApiClient::new();
    let profile: ProfileDTO = api
        .call("users/me", &Some(req.clone()), None::<&()>, Method::GET)
        .await
        .map_err(|err| {
            log::error!("users/me API error: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to get user data. Please try again." }),
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
        .user
        .unwrap()
        .roles
        .into_iter()
        .map(|r| r.pid)
        .collect::<Vec<Uuid>>();

    Ok(role_ids)
}

#[get("")]
async fn get_logged_in_user_data(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> Result<ApiResponse, ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;
    let profile = get_profile_data(&req).await?;

    let patient = main::entities::patients::Entity::find_by_sso_user_id(claims.sub)
        .filter(main::entities::patients::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!(
                "Failed to query patient by sso_user_id {}: {:?}",
                claims.sub,
                err
            );

            ApiResponse::new(
                500,
                json!({ "message": "Internal server error. Please try again later." }),
            )
        })?;

    let profile_picture = patient.as_ref().and_then(|p| p.photo_url.clone());

    let has_patient_data = patient.is_some();

    if let Some(user) = &profile.user {
        Ok(ApiResponse::new(
            200,
            json!({
                "profile": {
                    "pid": user.pid,
                    "first_name": &user.first_name,
                    "last_name": &user.last_name,
                    "email": &user.email,
                    "is_email_verified": user.is_email_verified,
                    "country_code": &user.country_code,
                    "phone_number": &user.phone_number,
                    "is_phone_verified": user.is_phone_verified,
                    "username": &user.username,
                    "last_login": &user.last_login,
                    "is_enabled": user.is_enabled,
                    "method": &user.method,
                    "is_secret_verified": user.is_secret_verified,
                    "tenant_name": &user.tenant_name,
                    "roles": &user.roles,
                    "profile_picture": profile_picture,
                    "has_patient_data": has_patient_data,
                    "created_at": &user.created_at,
                },
                "message": "User profile fetched successfully"
            }),
        ))
    } else {
        Ok(ApiResponse::new(404, json!({"message": "User not found"})))
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct TotpResponse {
    pub otpauth_url: Option<String>,
    pub secret: Option<String>,
    pub message: String,
}

#[post("/totp")]
async fn enable_2fa_totp(req: HttpRequest) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();

    let totp: TotpResponse = api
        .call(
            "users/me/totp",
            &Some(req.clone()),
            None::<&()>,
            Method::POST,
        )
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
        return Ok(ApiResponse::new(
            400,
            json!({
                "message": totp.message
            }),
        ));
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
        .call(
            "users/me/totp",
            &Some(req.clone()),
            Some(&data),
            Method::PATCH,
        )
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
        return Ok(ApiResponse::new(
            400,
            json!({
                "message": totp.message
            }),
        ));
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
        .call(
            "users/me/verify_totp",
            &Some(req.clone()),
            Some(&data),
            Method::POST,
        )
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
        return Ok(ApiResponse::new(
            400,
            json!({
                "message": totp.message
            }),
        ));
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
        .call(
            "users/me/regenerate_codes",
            &Some(req.clone()),
            Some(&data),
            Method::POST,
        )
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
        return Ok(ApiResponse::new(
            400,
            json!({
                "message": totp.message
            }),
        ));
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
async fn enable_otp(req: HttpRequest) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();

    let otp: ApiResponseDTO<()> = api
        .call(
            "users/me/enable_otp",
            &Some(req.clone()),
            None::<&()>,
            Method::POST,
        )
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
