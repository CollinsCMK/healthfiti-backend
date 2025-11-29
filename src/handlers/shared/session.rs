use actix_web::{HttpRequest, get};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::utils::{api_response::ApiResponse, http_client::ApiClient};

#[derive(Debug, Serialize, Deserialize)]
struct SessionResponse {
    message: String,
    data: Option<Vec<SessionItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionItem {
    refresh_data: RefreshTokenData,
    device_info: DeviceInfo,
    login_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenData {
    jti: String,
    user_pid: Uuid,
    tenant_pid: Option<Uuid>,
    device_id: Uuid,
    expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceInfo {
    ip_address: String,
    user_agent: String,
}

#[get("/me")]
async fn get_user_sessions(
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();

    let sessions: SessionResponse = api
        .call("users/me/sessions", &req, None::<&()>, Method::GET)
        .await
        .map_err(|err| {
            log::error!("users/me/sessions API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to get user sessions. Please try again."
                }),
            )
        })?;

    if sessions.data.is_none() {
        return Ok(ApiResponse::new(
            400,
            json!({
                "message": sessions.message
            }),
        ));
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": sessions.message,
            "sessions": sessions.data
        }),
    ))
}
