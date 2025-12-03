use actix_web::web;

use crate::utils::{api_response::ApiResponse, app_state::AppState};

pub async fn index(app_state: web::Data<AppState>) -> Result<ApiResponse, ApiResponse> {
    Ok(())
}
