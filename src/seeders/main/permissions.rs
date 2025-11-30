use reqwest::Method;
use serde::Deserialize;
use serde_json::json;

use crate::utils::{api_response::ApiResponse, http_client::ApiClient};

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct PermissionResponse {
    message: String,
}

pub async fn seed_permissions() -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();

    log::info!("SSO Base URL: {}", api.base_url);

    // name (snake_case), description, module
    let default_permissions = vec![
        // Patient Insurances
        (
            "view_all_patient_insurances",
            "Allows the user to view all patient insurance records in the system",
            "Patient Insurances",
        )
    ];

    // Convert default_permissions to Vec of objects expected by /create API
    let permissions_json: Vec<_> = default_permissions
        .into_iter()
        .map(|(name, description, module)| {
            json!({
                "name": name,
                "description": description,
                "module": module
            })
        })
        .collect();

    let request_json = json!(permissions_json);

    log::info!("Calling SSO API: {}/permissions/create", api.base_url);
    log::debug!("Request payload: {:?}", request_json);
    
    let response: PermissionResponse = api
        .call_with_secret("permissions/create", Some(&request_json), Method::POST)
        .await
        .map_err(|err| {
            log::error!("Permissions API error: {}", err);
            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to seed permissions"
                }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": response.message }),
    ))
}
