use actix_multipart::Multipart;
use actix_web::web;
use futures::StreamExt;
use serde_json::json;
use uuid::Uuid;

use crate::utils::{
    api_response::ApiResponse,
    app_state::AppState,
    multipart::{field_to_byte, get_presigned_url, upload_file},
};

async fn upload(
    mut payload: Multipart,
    app_state: web::Data<AppState>,
) -> Result<ApiResponse, ApiResponse> {
    let mut uploaded_files = Vec::new();

    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition().unwrap();
        let name = content_disposition.get_name().unwrap_or("");
        let filename = content_disposition
            .get_filename()
            .map(|f| f.to_string())
            .unwrap_or_else(|| format!("{}.bin", Uuid::new_v4()));
        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        match name {
            "files[]" => {
                let file_data = field_to_byte(&mut field).await?;

                if !file_data.is_empty() {
                    let unique_filename = format!("{}-{}", Uuid::new_v4(), filename);

                    upload_file(
                        &app_state,
                        &unique_filename,
                        file_data.clone(),
                        &content_type,
                    )
                    .await?;

                    let url = get_presigned_url(&app_state, &unique_filename, 3600).await?;

                    uploaded_files.push(json!({
                        "original_name": filename,
                        "stored_name": unique_filename,
                        "size": file_data.len(),
                        "content_type": content_type,
                        "url": url
                    }));
                }
            }
            _ => {}
        }
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": "File uploaded successfully",
            "files": uploaded_files
        }),
    ))
}
