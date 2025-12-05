use actix_web::{web, HttpRequest};
use uuid::Uuid;

use crate::{
    db::main,
    handlers::services::tenants::{
        TenantData, TenantStatus, create_tenant, destroy_tenant, edit_tenant, get_all_tenants,
        get_tenant_by_id, permanently_delete_tenant, restore_tenant, set_active_status_tenant,
    },
    utils::{api_response::ApiResponse, app_state::AppState, pagination::PaginationParams},
};

pub async fn index(query: web::Query<PaginationParams>) -> Result<ApiResponse, ApiResponse> {
    get_all_tenants(&query).await
}

pub async fn show(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let tenant_id = path.into_inner();
    let stmt = main::entities::tenants::Entity::find_by_pid(tenant_id);

    get_tenant_by_id(stmt, &app_state, tenant_id).await
}

pub async fn create(
    app_state: web::Data<AppState>,
    data: web::Json<TenantData>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    create_tenant(&app_state, &data, &req).await
}

pub async fn update(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    data: web::Json<TenantData>,
) -> Result<ApiResponse, ApiResponse> {
    let tenant_id = path.into_inner();

    edit_tenant(&app_state, tenant_id, &data).await
}

pub async fn set_active_status(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    data: web::Json<TenantStatus>,
) -> Result<ApiResponse, ApiResponse> {
    let tenant_id = path.into_inner();

    set_active_status_tenant(&app_state, tenant_id, &data).await
}

pub async fn destroy(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let tenant_id = path.into_inner();

    destroy_tenant(&app_state, tenant_id).await
}

pub async fn restore(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let tenant_id = path.into_inner();

    restore_tenant(&app_state, tenant_id).await
}

pub async fn delete_permanently(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let tenant_id = path.into_inner();

    permanently_delete_tenant(&app_state, tenant_id).await
}
