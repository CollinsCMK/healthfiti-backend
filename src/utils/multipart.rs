use std::str::FromStr;

use actix_multipart::Field;
use actix_web::web;
use aws_sdk_s3::{presigning::PresigningConfig, primitives::ByteStream};
use chrono::{NaiveDate, NaiveDateTime};
use futures::TryStreamExt;
use sea_orm::prelude::Decimal;
use serde_json::json;

use crate::utils::{api_response::ApiResponse, app_state::AppState};

pub async fn upload_file(
    app_state: &web::Data<AppState>,
    file_name: &str,
    content: Vec<u8>,
    content_type: &str,
) -> Result<(), ApiResponse> {
    app_state
        .s3_client
        .put_object()
        .bucket(&app_state.bucket)
        .key(file_name)
        .body(ByteStream::from(content))
        .content_type(content_type)
        .send()
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    Ok(())
}

pub async fn download_file(
    app_state: &web::Data<AppState>,
    file_name: &str,
) -> Result<Vec<u8>, ApiResponse> {
    let response = app_state
        .s3_client
        .get_object()
        .bucket(&app_state.bucket)
        .key(file_name)
        .send()
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    let data = response
        .body
        .collect()
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    Ok(data.to_vec())
}

pub async fn get_presigned_url(
    app_state: &web::Data<AppState>,
    file_name: &str,
    expiry_secs: u64,
) -> Result<String, ApiResponse> {
    let presigning_config =
        PresigningConfig::expires_in(std::time::Duration::from_secs(expiry_secs))
            .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    let presigned_request = app_state
        .s3_client
        .get_object()
        .bucket(&app_state.bucket)
        .key(file_name)
        .presigned(presigning_config)
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    Ok(presigned_request.uri().to_string())
}

pub async fn delete_file(
    app_state: &web::Data<AppState>,
    file_name: &str,
) -> Result<(), ApiResponse> {
    app_state
        .s3_client
        .delete_object()
        .bucket(&app_state.bucket)
        .key(file_name)
        .send()
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    Ok(())
}

async fn read_field_data(field: &mut Field) -> Result<Vec<u8>, ApiResponse> {
    let mut data = Vec::new();
    while let Some(chunk) = field
        .try_next()
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?
    {
        data.extend_from_slice(&chunk);
    }
    Ok(data)
}

async fn field_to_string_internal(field: &mut Field) -> Result<String, ApiResponse> {
    let data = read_field_data(field).await?;
    String::from_utf8(data)
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))
}

pub async fn field_to_decimal(field: &mut Field) -> Result<Decimal, ApiResponse> {
    let data_str = field_to_string_internal(field).await?;
    let trimmed = data_str.trim();

    if trimmed.is_empty() {
        return Ok(Decimal::new(0, 0));
    }

    Decimal::from_str(trimmed)
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))
}

pub async fn field_to_f64(field: &mut Field) -> Result<f64, ApiResponse> {
    let data_str = field_to_string_internal(field).await?;
    data_str
        .trim()
        .parse::<f64>()
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))
}

pub async fn field_to_vec_string(field: &mut Field) -> Result<Vec<String>, ApiResponse> {
    let data_str = field_to_string_internal(field).await?;
    Ok(data_str.split(',').map(|s| s.trim().to_string()).collect())
}

pub async fn field_to_vec_i32(field: &mut Field) -> Result<Vec<i32>, ApiResponse> {
    let data_str = field_to_string_internal(field).await?;
    Ok(data_str
        .split(',')
        .filter_map(|s| s.trim().parse::<i32>().ok())
        .collect())
}

pub async fn field_to_string(field: &mut Field) -> Result<String, ApiResponse> {
    field_to_string_internal(field).await
}

pub async fn field_to_bool(field: &mut Field) -> Result<bool, ApiResponse> {
    let value = field_to_string_internal(field).await?;
    Ok(matches!(value.trim().to_lowercase().as_str(), "true" | "1"))
}

pub async fn field_to_datetime(field: &mut Field) -> Result<NaiveDateTime, ApiResponse> {
    let date_string = field_to_string_internal(field).await?;
    NaiveDateTime::parse_from_str(date_string.trim(), "%Y-%m-%d %H:%M:%S")
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))
}

pub async fn field_to_date(field: &mut Field) -> Result<NaiveDate, ApiResponse> {
    let date_string = field_to_string_internal(field).await?;
    NaiveDate::parse_from_str(date_string.trim(), "%Y-%m-%d")
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))
}

pub async fn field_to_byte(field: &mut Field) -> Result<Vec<u8>, ApiResponse> {
    read_field_data(field).await
}

pub async fn field_to_json(field: &mut Field) -> Result<serde_json::Value, ApiResponse> {
    let string_data = field_to_string_internal(field).await?;
    serde_json::from_str(&string_data)
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))
}
