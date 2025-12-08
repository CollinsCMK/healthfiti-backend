use std::collections::HashMap;

use actix_web::{HttpRequest, patch, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    handlers::services::tenants::ApiResponseDTO,
    utils::{
        api_response::ApiResponse,
        http_client::ApiClient,
        validation::{validate_password, validate_phone_number},
        validator_error::ValidationError,
    },
};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
struct ChangeEmailData {
    password: String,
    email: String,
    domain: Option<String>,
}

impl ChangeEmailData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.password.trim().is_empty() {
            errors.insert("password".into(), "Current password is required.".into());
        }

        if self.email.trim().is_empty() {
            errors.insert("email".into(), "Email address is required.".into());
        } else if !self.email.trim().contains('@') {
            errors.insert(
                "email".into(),
                "Please provide a valid email address.".into(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[patch("/email")]
async fn change_email(
    data: web::Json<ChangeEmailData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!({ "message": err })));
    }

    let api = ApiClient::new();
    let email: ApiResponseDTO<()> = api
        .call(
            "users/me/email",
            &Some(req.clone()),
            None::<&()>,
            Method::PATCH,
        )
        .await
        .map_err(|err| {
            log::error!("users/me/email API error: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to update user email. Please try again." }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": email.message
        }),
    ))
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
struct ChangePhoneData {
    password: String,
    country_code: String,
    phone_number: String,
    domain: Option<String>,
}

impl ChangePhoneData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.password.trim().is_empty() {
            errors.insert("password".into(), "Current password is required.".into());
        }

        if self.country_code.trim().is_empty() {
            errors.insert("country_code".into(), "Country code is required.".into());
        }

        if self.phone_number.trim().is_empty() {
            errors.insert("phone_number".into(), "Phone number is required.".into());
        } else if !validate_phone_number(&self.phone_number) {
            errors.insert("phone_number".into(), "Invalid phone number format".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[patch("/phone")]
async fn change_phone_number(
    data: web::Json<ChangePhoneData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!({ "message": err })));
    }

    let api = ApiClient::new();
    let phone: ApiResponseDTO<()> = api
        .call(
            "users/me/phone",
            &Some(req.clone()),
            None::<&()>,
            Method::PATCH,
        )
        .await
        .map_err(|err| {
            log::error!("users/me/phone API error: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to update user phone. Please try again." }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": phone.message
        }),
    ))
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
struct PasswordData {
    current_password: String,
    password: String,
    confirm_password: String,
    domain: Option<String>,
}

impl PasswordData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.current_password.trim().is_empty() {
            errors.insert(
                "current_password".into(),
                "Current password is required.".into(),
            );
        }

        if self.password.trim().is_empty() {
            errors.insert("password".into(), "New password is required.".into());
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

#[patch("/password")]
async fn change_password(
    data: web::Json<PasswordData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!({ "message": err })));
    }

    let api = ApiClient::new();
    let password: ApiResponseDTO<()> = api
        .call(
            "users/me/password",
            &Some(req.clone()),
            None::<&()>,
            Method::PATCH,
        )
        .await
        .map_err(|err| {
            log::error!("users/me/password API error: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to update user password. Please try again." }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": password.message
        }),
    ))
}
