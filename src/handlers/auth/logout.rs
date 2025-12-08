use actix_web::{HttpRequest, post, web};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    handlers::{auth::refresh::RefreshRequest, services::tenants::ApiResponseDTO},
    utils::{api_response::ApiResponse, http_client::ApiClient},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutQuery {
    all: Option<bool>,
    jti: Option<String>,
}

#[post("/logout")]
pub async fn logout(
    data: web::Json<RefreshRequest>,
    query: web::Query<LogoutQuery>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    let logout_all = query.all.unwrap_or(false);
    let refresh_jti = query.jti.clone().unwrap_or_default();

    let mut endpoint = format!("auth/logout?all={}", logout_all);

    if !refresh_jti.is_empty() {
        endpoint.push_str(&format!("&jti={}", refresh_jti));
    }

    let api = ApiClient::new();

    let logout: ApiResponseDTO<()> = api
        .call(&endpoint, &Some(req.clone()), Some(&*data), Method::POST)
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
