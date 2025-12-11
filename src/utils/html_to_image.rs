use std::{fs, process::Command};

use actix_web::HttpRequest;
use serde_json::json;
use uuid::Uuid;

use crate::utils::{api_response::ApiResponse, app_state::AppState, multipart::upload_file};

pub async fn generate_receipt_png(
    html: &str,
    req: &HttpRequest,
    app_state: &AppState,
) -> Result<String, ApiResponse> {
    let html_path = "/tmp/invoice_temp.html";
    fs::write(html_path, html).map_err(|err| {
        log::error!("Failed to write HTML to temp file: {}", err);
        ApiResponse::new(500, json!({ "message": "Internal server error" }))
    })?;

    let tmp_png_path = format!("/tmp/{}.png", Uuid::new_v4());

    let output = Command::new("wkhtmltoimage")
        .args([
            "--format",
            "png",
            "--quality",
            "100",
            "--width",
            "650",
            "--enable-local-file-access",
            "--no-stop-slow-scripts",
            html_path,
            &tmp_png_path,
        ])
        .output()
        .map_err(|err| {
            log::error!("Failed to execute wkhtmltoimage: {}", err);
            ApiResponse::new(500, json!({ "message": "Internal server error" }))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(ApiResponse::new(
            500,
            json!({
                "error": format!("wkhtmltoimage execution failed:\nStderr: {}\nStdout: {}", stderr, stdout)
            }),
        ));
    }

    let file_bytes = fs::read(&tmp_png_path).map_err(|err| {
        log::error!("Failed to read generated PNG: {}", err);
        ApiResponse::new(500, json!({ "message": "Internal server error" }))
    })?;

    let s3_key = format!("invoice/{}.png", Uuid::new_v4());
    let s3_url = upload_file(req, app_state, &s3_key, file_bytes, "image/png").await?;

    Ok(s3_url)
}
