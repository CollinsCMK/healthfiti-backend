use actix_web::web;
use chrono::NaiveDateTime;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    handlers::{admin::users::UserData, services::tenants::ApiResponseDTO},
    utils::{api_response::ApiResponse, http_client::ApiClient, pagination::PaginationParams},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantUserDTO {
    pid: Option<Uuid>,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    country_code: Option<String>,
    phone_number: Option<String>,
    username: Option<String>,
    last_login: Option<NaiveDateTime>,
    is_active: bool,
    deleted_at: Option<NaiveDateTime>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

pub async fn index(query: web::Query<PaginationParams>) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();

    let mut endpoint = format!(
        "tenant_users?all={}&page={}&limit={}",
        query.all.unwrap_or(false),
        query.page.unwrap_or(1),
        query.limit.unwrap_or(10)
    );

    if let Some(term) = &query.search {
        endpoint.push_str(&format!("&search={}", term));
    }

    let response: ApiResponseDTO<Vec<TenantUserDTO>> = api
        .call(&endpoint, &None, None::<&()>, Method::GET)
        .await
        .map_err(|err| {
            log::error!("Error getting users: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to get users" }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "users": response.data.unwrap(),
            "page": response.pagination.as_ref().unwrap().page,
            "total_pages": response.pagination.as_ref().unwrap().total_pages,
            "total_items": response.pagination.as_ref().unwrap().total_items,
            "has_prev": response.pagination.as_ref().unwrap().has_prev,
            "has_next": response.pagination.as_ref().unwrap().has_next,
            "message": response.message,
        }),
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantUserResponse {
    pub pid: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub country_code: Option<String>,
    pub phone_number: Option<String>,
    pub username: Option<String>,
    pub last_login: Option<String>,
    pub is_active: Option<bool>,
    pub deleted_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub role_ids: Option<Vec<String>>,
}

pub async fn show(path: web::Path<String>) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();
    let endpoint = format!("tenant_users/show/{}", path.into_inner());
    let response: ApiResponseDTO<TenantUserResponse> = api
        .call(&endpoint, &None, None::<&()>, Method::GET)
        .await
        .map_err(|err| {
            log::error!("Error getting user: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to get user" }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "user": response.data.unwrap(),
            "message": response.message,
        }),
    ))
}

pub async fn create(data: web::Json<UserData>) -> Result<ApiResponse, ApiResponse> {
    let json_data = json!({
        "first_name": data.first_name,
        "last_name": data.last_name,
        "username": data.username,
        "email": data.email,
        "country_code": data.country_code,
        "phone_number": data.phone_number,
        "role_ids": data.role_ids,
        "application_id": data.application_id,
        "tenant_id": data.tenant_id,
        "is_active": data.is_active,
    });

    let api = ApiClient::new();

    let response: ApiResponseDTO<()> = api
        .call("tenant_users/create", &None, Some(&json_data), Method::POST)
        .await
        .map_err(|err| {
            log::error!("Error creating user: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to create user" }))
        })?;

    if let Some(errors) = &response.errors {
        return Err(ApiResponse::new(400, json!({ "errors": errors })));
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": response.message,
        }),
    ))
}

pub async fn edit(
    path: web::Path<String>,
    data: web::Json<UserData>,
) -> Result<ApiResponse, ApiResponse> {
    let json_data = json!({
        "first_name": data.first_name,
        "last_name": data.last_name,
        "username": data.username,
        "email": data.email,
        "country_code": data.country_code,
        "phone_number": data.phone_number,
        "role_ids": data.role_ids,
        "application_id": data.application_id,
        "tenant_id": data.tenant_id,
        "is_active": data.is_active,
    });

    let api = ApiClient::new();

    let response: ApiResponseDTO<()> = api
        .call(
            &format!("tenant_users/edit/{}", path.into_inner()),
            &None,
            Some(&json_data),
            Method::POST,
        )
        .await
        .map_err(|err| {
            log::error!("Error editing user: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to edit user" }))
        })?;

    if let Some(errors) = &response.errors {
        return Err(ApiResponse::new(400, json!({ "errors": errors })));
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": response.message,
        }),
    ))
}

pub async fn set_active_status(path: web::Path<String>) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();

    let response: ApiResponseDTO<()> = api
        .call(
            &format!("tenant_users/status/{}", path.into_inner()),
            &None,
            None::<&()>,
            Method::POST,
        )
        .await
        .map_err(|err| {
            log::error!("Error setting active status for user: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to set active status for user" }),
            )
        })?;

    if let Some(errors) = &response.errors {
        return Err(ApiResponse::new(400, json!({ "errors": errors })));
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": response.message,
        }),
    ))
}

pub async fn destroy(path: web::Path<String>) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();

    let response: ApiResponseDTO<()> = api
        .call(
            &format!("tenant_users/soft-delete/{}", path.into_inner()),
            &None,
            None::<&()>,
            Method::DELETE,
        )
        .await
        .map_err(|err| {
            log::error!("Error soft deleting user: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to soft delete user" }))
        })?;

    if let Some(errors) = &response.errors {
        return Err(ApiResponse::new(400, json!({ "errors": errors })));
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": response.message,
        }),
    ))
}

pub async fn restore(path: web::Path<String>) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();

    let response: ApiResponseDTO<()> = api
        .call(
            &format!("tenant_users/restore/{}", path.into_inner()),
            &None,
            None::<&()>,
            Method::POST,
        )
        .await
        .map_err(|err| {
            log::error!("Error restoring user: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to restore user" }))
        })?;

    if let Some(errors) = &response.errors {
        return Err(ApiResponse::new(400, json!({ "errors": errors })));
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": response.message,
        }),
    ))
}

pub async fn delete_permanently(path: web::Path<String>) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();

    let response: ApiResponseDTO<()> = api
        .call(
            &format!("tenant_users/permanent/{}", path.into_inner()),
            &None,
            None::<&()>,
            Method::DELETE,
        )
        .await
        .map_err(|err| {
            log::error!("Error deleting user: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to delete user" }))
        })?;

    if let Some(errors) = &response.errors {
        return Err(ApiResponse::new(400, json!({ "errors": errors })));
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": response.message,
        }),
    ))
}
