use crate::{
    handlers::services::tenants::ApiResponseDTO,
    utils::{
        api_response::ApiResponse, http_client::ApiClient, jwt::get_logged_in_user_claims,
        pagination::PaginationParams,
    },
};
use actix_web::HttpRequest;
use chrono::NaiveDateTime;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantApplication {
    pub pid: Option<String>,
    pub admin_subdomain: Option<String>,
    pub admin_domain: Option<String>,
    pub patient_subdomain: Option<String>,
    pub patient_domain: Option<String>,
    pub branding: Option<Value>,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

pub async fn get_all_tenant_applications(
    query: &PaginationParams,
) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();
    
    let mut endpoint = format!(
        "tenant_applications?all={}&page={}&limit={}",
        query.all.unwrap_or(false),
        query.page.unwrap_or(1),
        query.limit.unwrap_or(10)
    );

    if let Some(term) = &query.search {
        endpoint.push_str(&format!("&search={}", term));
    }

    let response: ApiResponseDTO<Vec<TenantApplication>> = api
        .call(&endpoint, &None, None::<&()>, Method::GET)
        .await
        .map_err(|err| {
            log::error!("Failed to find tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to find tenant" }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "tenant_applications": response.data.unwrap(),
            "page": response.pagination.as_ref().unwrap().page,
            "total_pages": response.pagination.as_ref().unwrap().total_pages,
            "total_items": response.pagination.as_ref().unwrap().total_items,
            "has_prev": response.pagination.as_ref().unwrap().has_prev,
            "has_next": response.pagination.as_ref().unwrap().has_next,
            "message": response.message,
        }),
    ))
}

pub async fn get_tenant_application_data(
    req: &HttpRequest,
) -> Result<ApiResponseDTO<TenantApplication>, ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;

    let api = ApiClient::new();
    let endpoint = format!(
        "tenant_applications/check/{}/{}",
        claims.tenant_pid.unwrap(),
        claims.application_pid.unwrap()
    );
    let response: ApiResponseDTO<TenantApplication> = api
        .call(&endpoint, &None, None::<&()>, Method::GET)
        .await
        .map_err(|err| {
            log::error!("Failed to find tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to find tenant" }))
        })?;

    Ok(response)
}

pub async fn get_tenant_application_by_id(pid: Uuid) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();
    let endpoint = format!("tenant_applications/show/{}", pid);
    let response: ApiResponseDTO<TenantApplication> = api
        .call(&endpoint, &None, None::<&()>, Method::GET)
        .await
        .map_err(|err| {
            log::error!("Failed to find tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to find tenant" }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "tenant_application": response.data.unwrap(),
            "message": response.message,
        }),
    ))
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TenantApplicationData {
    pub application_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub admin_subdomain: String,
    pub admin_domain: Option<String>,
    pub patient_subdomain: String,
    pub patient_domain: Option<String>,
    pub branding: Option<Value>,
}

#[derive(Deserialize, Debug)]
pub struct CreateTenantApplicationResponse {
    pub pid: Uuid,
}

pub async fn create_tenant_application(
    data: TenantApplicationData,
) -> Result<ApiResponse, ApiResponse> {
    let json_value = json!({
        "application_id": data.application_id,
        "tenant_id": data.tenant_id,
        "admin_subdomain": data.admin_subdomain,
        "admin_domain": data.admin_domain,
        "patient_subdomain": data.patient_subdomain,
        "patient_domain": data.patient_domain,
        "branding": data.branding,
    });

    let api = ApiClient::new();
    let response: ApiResponseDTO<CreateTenantApplicationResponse> = api
        .call(
            "tenant_applications/create",
            &None,
            Some(&json_value),
            Method::POST,
        )
        .await
        .map_err(|err| {
            log::error!("Failed to create tenant application: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to create tenant application" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "pid": response.data.unwrap().pid,
            "message": response.message,
        }),
    ))
}

pub async fn update_tenant_application(
    pid: Uuid,
    data: TenantApplicationData,
) -> Result<ApiResponse, ApiResponse> {
    let json_value = json!({
        "application_id": data.application_id,
        "tenant_id": data.tenant_id,
        "admin_subdomain": data.admin_subdomain,
        "admin_domain": data.admin_domain,
        "patient_subdomain": data.patient_subdomain,
        "patient_domain": data.patient_domain,
        "branding": data.branding,
    });

    let api = ApiClient::new();
    let response: ApiResponseDTO<()> = api
        .call(
            &format!("tenant_applications/edit/{}", pid),
            &None,
            Some(&json_value),
            Method::PUT,
        )
        .await
        .map_err(|err| {
            log::error!("Failed to update tenant application: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to update tenant application" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": response.message,
        }),
    ))
}

pub async fn destroy_tenant_application(pid: Uuid) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();
    let response: ApiResponseDTO<()> = api
        .call(
            &format!("tenant_applications/soft-delete/{}", pid),
            &None,
            None::<&()>,
            Method::DELETE,
        )
        .await
        .map_err(|err| {
            log::error!("Failed to destroy tenant application: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to destroy tenant application" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": response.message,
        }),
    ))
}

pub async fn restore_tenant_application(pid: Uuid) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();
    let response: ApiResponseDTO<()> = api
        .call(
            &format!("tenant_applications/restore/{}", pid),
            &None,
            None::<&()>,
            Method::PATCH,
        )
        .await
        .map_err(|err| {
            log::error!("Failed to restore tenant application: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to restore tenant application" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": response.message,
        }),
    ))
}

pub async fn delete_tenant_application(pid: Uuid) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();
    let response: ApiResponseDTO<()> = api
        .call(
            &format!("tenant_applications/delete/{}", pid),
            &None,
            None::<&()>,
            Method::DELETE,
        )
        .await
        .map_err(|err| {
            log::error!("Failed to delete tenant application: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to delete tenant application" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": response.message,
        }),
    ))
}
