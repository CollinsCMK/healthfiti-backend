use actix_web::{post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    handlers::auth::{phone_verification::SuccessResponse, refresh::RefreshRequest},
    utils::{
        api_response::ApiResponse,
        http_client::{ApiClient, EndpointType},
    },
};

#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutQuery {
    all: Option<bool>,
}

#[post("/logout")]
pub async fn logout(
    data: web::Json<RefreshRequest>,
    query: web::Query<LogoutQuery>,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    let logout_all = query.all.unwrap_or(false);
    let endpoint = format!("auth/logout?all={}", logout_all);

    let api = ApiClient::new();

    let logout: SuccessResponse = api
        .call(&endpoint, EndpointType::Auth, Some(&*data), Method::POST)
        .await
        .map_err(|err| {
            log::error!("Logout API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to logout. Please try again."
                }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": logout.message
        }),
    ))
}
