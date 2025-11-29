use std::collections::HashMap;

use actix_web::{get, web};
use serde_json::json;

use crate::utils::{api_response::ApiResponse, app_state::AppState, multipart::get_presigned_url};

#[get("/files/download")]
async fn get_file_url(
    app_state: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
) -> Result<ApiResponse, ApiResponse> {
    let file_name = query
        .get("file_name")
        .ok_or_else(|| ApiResponse::new(400, json!({ "message": "file_name missing" })))?;

    let url = get_presigned_url(&app_state, file_name, 3600).await?;

    Ok(ApiResponse::new(
        200,
        json!({
            "url": url
        }),
    ))
}
