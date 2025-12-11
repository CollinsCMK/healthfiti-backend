use actix_web::{HttpRequest, post, web};
use chrono::{Datelike, Utc};
use serde::Deserialize;
use serde_json::json;

use crate::{
    emails::test::send_test_email,
    handlers::shared::profile::get_profile_data,
    utils::{api_response::ApiResponse, app_state::AppState, message_queue::MessageType},
};

#[derive(Deserialize)]
struct SendTestRequest {
    domain: Option<String>,
}

#[post("")]
async fn send(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    data: web::Json<SendTestRequest>,
) -> Result<ApiResponse, ApiResponse> {
    let profile = get_profile_data(&req).await.map_err(|err| {
        ApiResponse::new(
            500,
            json!({
                "message": err.to_string()
            }),
        )
    })?;

    if let Some(user) = &profile.user {
        let phone_number = format!("{}{}", user.country_code.trim(), user.phone_number.trim());

        app_state
            .message_queue
            .send_message(MessageType::SMS {
                phone_number: phone_number,
                message: "This is a test SMS to confirm that your phone number works.".to_string(),
            })
            .await
            .map_err(|e| ApiResponse::new(500, json!({ "message": e })))?;

        send_test_email(
            user.email.to_string(),
            &user.username,
            Utc::now().year(),
            &app_state.clone(),
            data.domain.clone(),
            None,
            None,
        )
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

        return Ok(ApiResponse::new(
            200,
            json!({
                "message": "Test SMS and email sent successfully"
            }),
        ));
    } else {
        return Err(ApiResponse::new(
            404,
            json!({ "message": "User not found" }),
        ));
    }
}
