use std::collections::HashMap;

use actix_web::{HttpRequest, post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils::{
    api_response::ApiResponse, http_client::ApiClient, validator_error::ValidationError,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshRequest {
    #[serde(default)]
    pub refresh_token: String,
}

impl RefreshRequest {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.refresh_token.trim().is_empty() {
            errors.insert(
                "refresh_token".to_string(),
                "Refresh token is required".to_string(),
            );
        }

        if !errors.is_empty() {
            return Err(ValidationError { errors });
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct RefreshSuccessData {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in_seconds: i64,
}

#[derive(Deserialize, Debug)]
struct RefreshResponse {
    #[serde(default)]
    data: Option<RefreshSuccessData>,
    message: String,
}

#[post("/refresh")]
pub async fn refresh(
    data: web::Json<RefreshRequest>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    let api = ApiClient::new();

    let refresh: RefreshResponse = api
        .call(
            "auth/refresh",
            &Some(req.clone()),
            Some(&*data),
            Method::POST,
        )
        .await
        .map_err(|err| {
            log::error!("Refresh API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to refresh token. Please try again."
                }),
            )
        })?;

    if refresh.data.is_none() {
        return Err(ApiResponse::new(
            500,
            json!({
                "message": refresh.message
            }),
        ));
    }

    Ok(ApiResponse::new(
        200,
        json!({
                "data": refresh.data,
            "message": refresh.message
        }),
    ))
}
