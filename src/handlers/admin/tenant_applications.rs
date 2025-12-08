use actix_web::web;
use uuid::Uuid;

use crate::{
    handlers::services::tenant_applications::{
        TenantApplicationData, create_tenant_application, delete_tenant_application,
        destroy_tenant_application, get_all_tenant_applications, get_tenant_application_by_id,
        get_tenant_application_data, restore_tenant_application, update_tenant_application,
    },
    utils::{api_response::ApiResponse, pagination::PaginationParams},
};

pub async fn index(query: web::Query<PaginationParams>) -> Result<ApiResponse, ApiResponse> {
    get_all_tenant_applications(&query).await
}

pub async fn show(path: web::Path<Uuid>) -> Result<ApiResponse, ApiResponse> {
    get_tenant_application_by_id(path.into_inner()).await
}

// pub async fn check(path: web::Path<Uuid>) -> Result<ApiResponse, ApiResponse> {
//     let tenant_application_id = path.into_inner();
//     get_tenant_application_data(tenant_application_id).await
// }

pub async fn create(req: web::Json<TenantApplicationData>) -> Result<ApiResponse, ApiResponse> {
    create_tenant_application(req.into_inner()).await
}

pub async fn update(
    path: web::Path<Uuid>,
    req: web::Json<TenantApplicationData>,
) -> Result<ApiResponse, ApiResponse> {
    update_tenant_application(path.into_inner(), req.into_inner()).await
}

pub async fn destroy(path: web::Path<Uuid>) -> Result<ApiResponse, ApiResponse> {
    destroy_tenant_application(path.into_inner()).await
}

pub async fn restore(path: web::Path<Uuid>) -> Result<ApiResponse, ApiResponse> {
    restore_tenant_application(path.into_inner()).await
}

pub async fn delete_permanently(path: web::Path<Uuid>) -> Result<ApiResponse, ApiResponse> {
    delete_tenant_application(path.into_inner()).await
}
