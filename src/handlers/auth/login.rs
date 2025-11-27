use std::collections::HashMap;

use actix_web::{post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    db::tenant::{self, ActiveModelTrait, ActiveValue::Set, ColumnTrait, QueryFilter},
    utils::{
        api_response::ApiResponse,
        app_state::AppState,
        http_client::{ApiClient, EndpointType},
        validation::validate_phone_number,
        validator_error::ValidationError,
    },
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
    #[serde(default)]
    tenant_pid: Option<String>,
    message: String,
}

async fn login(data: &web::Json<LoginData>, user_type: &str) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    let api = ApiClient::new();

    let request_json = json!({
        "credential": data.credential,
        "country_code": data.country_code,
        "phone_number": data.phone_number,
        "password": data.password,
        "domain": data.domain,
        "user_type": user_type,
    });

    let login: LoginResponse = api
        .call(
            "auth/login",
            EndpointType::Auth,
            Some(&request_json),
            Method::POST,
        )
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

    if login.is_verify_email.unwrap_or(false) {
        return Ok(ApiResponse::new(
            200,
            json!({
                "is_verify_email": true,
                "user_id": login.user_id.expect("User ID"),
                "tenant_pid": login.tenant_pid,
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
                "tenant_pid": login.tenant_pid,
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
                "tenant_pid": login.tenant_pid,
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
                "tenant_pid": login.tenant_pid,
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

#[post("/admin/login")]
async fn admin_login(data: web::Json<LoginData>) -> Result<ApiResponse, ApiResponse> {
    match login(&data, "admin").await {
        Ok(response) => Ok(response),
        Err(err) => {
            log::error!("Admin login failed: {:?}", err);

            Err(ApiResponse::new(
                500,
                json!({ "message": "Internal server error" }),
            ))
        }
    }
}

#[post("/tenant/login")]
async fn tenant_login(
    data: web::Json<LoginData>,
    app_state: web::Data<AppState>,
) -> Result<ApiResponse, ApiResponse> {
    let login_response = match login(&data, "tenant").await {
        Ok(resp) => resp,
        Err(err) => {
            log::error!("Tenant login failed before tenant DB lookup: {:?}", err);

            return Err(ApiResponse::new(
                500,
                json!({ "message": "Internal server error" }),
            ));
        }
    };

    let body_json: Value = serde_json::from_str(&login_response.body).map_err(|err| {
        log::error!("Failed to parse login_response body: {}", err);
        ApiResponse::new(500, json!({ "message": "Invalid response from SSO" }))
    })?;

    let sso_user_id_str = body_json
        .get("data")
        .and_then(|d| d.get("user_id"))
        .and_then(|id| id.as_str())
        .ok_or_else(|| {
            ApiResponse::new(500, json!({ "message": "SSO response missing user_id" }))
        })?;

    let sso_user_id = Uuid::parse_str(sso_user_id_str).map_err(|err| {
        log::error!("Failed to parse sso_user_id as UUID: {}", err);
        ApiResponse::new(500, json!({ "message": "Invalid SSO user_id format" }))
    })?;

    let sso_tenant_id_str = body_json
        .get("data")
        .and_then(|d| d.get("tenant_pid"))
        .and_then(|id| id.as_str())
        .ok_or_else(|| {
            ApiResponse::new(500, json!({ "message": "SSO response missing tenant_id" }))
        })?;

    let sso_tenant_id = Uuid::parse_str(sso_tenant_id_str).map_err(|err| {
        log::error!("Failed to parse sso_tenant_id as UUID: {}", err);
        ApiResponse::new(500, json!({ "message": "Invalid SSO tenant_id format" }))
    })?;

    let tenant_db = match app_state.tenant_db(sso_tenant_id) {
        Some(db) => db,
        None => {
            log::error!("Tenant DB not found for tenat_id: {}", sso_tenant_id);
            return Err(ApiResponse::new(
                404,
                json!({ "message": "Tenant database not found" }),
            ));
        }
    };

    let existing_user = tenant::entities::users::Entity::find_by_sso_user_id(sso_user_id.clone())
        .filter(tenant::entities::users::Column::DeletedAt.is_null())
        .one(&tenant_db)
        .await
        .map_err(|err| {
            log::error!("DB error during tenant lookup: {}", err);
            ApiResponse::new(500, json!({ "message": "Database error" }))
        })?;

    let _tenant_user = match existing_user {
        Some(user) => user,
        None => {
            log::info!("Tenant user not found. Creating new tenant user...");

            tenant::entities::users::ActiveModel {
                sso_user_id: Set(sso_user_id.clone()),
                ..Default::default()
            }
            .insert(&tenant_db)
            .await
            .map_err(|err| {
                log::error!("Failed to create tenant user: {}", err);
                ApiResponse::new(500, json!({ "message": "Failed to create tenant user" }))
            })?
        }
    };

    Ok(login_response)
}

#[post("/normal/login")]
async fn normal_login(data: web::Json<LoginData>) -> Result<ApiResponse, ApiResponse> {
    match login(&data, "normal").await {
        Ok(response) => Ok(response),
        Err(err) => {
            log::error!("Normal login failed: {:?}", err);

            Err(ApiResponse::new(
                500,
                json!({ "message": "Internal server error" }),
            ))
        }
    }
}
