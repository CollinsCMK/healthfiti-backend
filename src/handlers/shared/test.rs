use actix_web::{HttpRequest, web};
use chrono::{Datelike, Utc};
use serde_json::json;

use crate::{emails::test::send_test_email, handlers::shared::profile::get_profile_data, utils::{api_response::ApiResponse, app_state::AppState, message_queue::MessageType}};

pub async fn send(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let profile = get_profile_data(&req).await.map_err(|err| {
        ApiResponse::new(
            500,
            json!({
                "message": err.to_string()
            }),
        )
    })?;

    let phone_number = format!("{}{}", profile.country_code.trim(), profile.phone_number.trim());

    app_state
        .message_queue
        .send_message(MessageType::SMS {
            phone_number: phone_number,
            message: "This is a test SMS to confirm that your phone number works.".to_string(),
        })
        .await
        .map_err(|e| ApiResponse::new(500, json!({ "message": e })))?;

    send_test_email(
        profile.email.to_string(),
        &profile.username,
        Utc::now().year(),
        app_state.clone(),
    )
    .await
    .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    Ok(ApiResponse::new(200, json!({
        "message": "Test SMS and email sent successfully"
    })))
}