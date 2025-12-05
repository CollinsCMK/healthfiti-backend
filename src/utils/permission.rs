use actix_web::HttpRequest;
use reqwest::Method;
use serde::Deserialize;
use serde_json::json;

use crate::utils::{api_response::ApiResponse, http_client::ApiClient};

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct ExtractPermissionResponse {
    has_permission: bool,
    message: String,
}

pub async fn has_permission(permission_name: &str, req: &HttpRequest) -> Result<bool, ApiResponse> {
    let payload = json!({ "permission_name": permission_name });

    let api = ApiClient::new();

    let response: ExtractPermissionResponse = api
        .call(
            "permissions/check",
            &Some(req.clone()),
            Some(&payload),
            Method::POST,
        )
        .await
        .map_err(|err| {
            log::error!("Permission check API error: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to check permission" }))
        })?;

    Ok(response.has_permission)
}

pub async fn extract_permissions(
    guard_name: String,
    req: &HttpRequest,
) -> Result<bool, ApiResponse> {
    let payload = json!({ "guard_name": guard_name });

    let api = ApiClient::new();

    let response: ExtractPermissionResponse = api
        .call(
            "permissions/list",
            &Some(req.clone()),
            Some(&payload),
            Method::POST,
        )
        .await
        .map_err(|err| {
            log::error!("Permission list API error: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to retrieve permissions" }))
        })?;

    Ok(response.has_permission)
}
