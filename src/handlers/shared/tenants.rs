use actix_web::{HttpRequest, get, web};
use reqwest::Method;
use serde_json::json;

use crate::{
    handlers::services::tenants::ApiResponseDTO, utils::{api_response::ApiResponse, http_client::ApiClient},
};

#[get("/check")]
async fn is_tenant_name_taken(
    req: HttpRequest,
    query: web::Query<String>,
) -> Result<ApiResponse, ApiResponse> {
    let name = query.into_inner();
    let endpoint = format!("tenants/check?tenant_name={}", name);

    let api = ApiClient::new();

    let tenant: ApiResponseDTO<()> = api
        .call(&endpoint, &Some(req.clone()), None::<&()>, Method::POST)
        .await
        .map_err(|err| {
            log::error!("tenants/check?tenant_name={} API error: {}", name, err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Tenant check failed. Please try again."
                }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": tenant.message,
        }),
    ))
}
