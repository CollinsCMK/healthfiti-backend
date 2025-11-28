use actix_web::web;
use serde_json::json;

use crate::{
    utils::{api_response::ApiResponse, app_state::AppState, message_queue::MessageType},
};

pub async fn send_test_email(
    to: String,
    username: &str,
    year: i32,
    app_state: web::Data<AppState>,
) -> Result<(), ApiResponse> {
    let email_logo = "DEFAULT_LOGO_URL";
    let email_privacy = "DEFAULT_PRIVACY_URL";

    let html = format!(
        r#"
    <!doctype html>
    <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
        </head>
        <body style="padding: 0px; margin: 0px; box-sizing: border-box; font-family: Arial, sans-serif;">
            <div style="background-color: #9C8F6280; padding: 4px; min-height: 100vh;">
                <div style="background-color: white; border-radius: 10px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); max-width: 600px; margin: 0 auto;">
                    <header style="display: flex; align-items: center; justify-content: center; background-color: #840C0180; width: 100%; padding: 20px 0; border-radius: 10px 10px 0 0;">
                        <img src="{}" alt="logo" style="width: 200px; height: auto; filter: drop-shadow(2px 2px 4px rgba(0,0,0,0.2));">
                    </header>

                    <div style="padding: 30px 20px;">
                        <h1 style="color: #840C01; text-align: center; margin-bottom: 30px; font-size: 28px;">[TEST EMAIL]</h1>

                        <p style="color: #333; font-size: 16px; margin-bottom: 15px;">Hello {},</p>

                        <p style="color: #444; font-size: 16px; line-height: 1.6; margin-bottom: 25px;">
                            This is a <strong>test email</strong> to confirm that your email address works correctly.
                        </p>

                        <p style="color: #888; font-size: 14px;">
                            If you received this email, your inbox is properly set up.
                        </p>
                    </div>
                </div>

                <div style="flex-grow: 1"></div>

                <footer style="text-align: center; font-size: 14px; color: #666; margin-top: 20px;">
                    This message was produced for testing purposes. <span>&copy; {}</span>. All rights reserved.
                    <br>
                    View our <a href="{}" style="cursor: pointer; color: #840C01; text-decoration: none; font-weight: bold;">privacy policy</a>.
                </footer>
            </div>
        </body>
    </html>
    "#,
        email_logo, username, year, email_privacy
    );

    app_state
        .message_queue
        .send_message(MessageType::Email {
            to,
            subject: "[TEST] Email Verification".to_string(),
            html,
        })
        .await
        .map_err(|e| ApiResponse::new(500, json!({ "message": e })))?;

    Ok(())
}