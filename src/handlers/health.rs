use actix_web::get;
use serde_json::json;

use crate::utils::api_response::ApiResponse;

#[get("/health")]
async fn health() -> Result<ApiResponse, ApiResponse> {
    Ok(ApiResponse::new(
        200,
        json!({
            "status": "ok",
            "message": "System is online"
        }),
    ))
}
