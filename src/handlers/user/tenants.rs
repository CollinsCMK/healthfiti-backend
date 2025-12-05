use crate::{
    db::main,
    handlers::services::tenants::{get_all_tenants, get_tenant_by_id},
    utils::{api_response::ApiResponse, app_state::AppState, pagination::PaginationParams},
};
use actix_web::{get, web};
use uuid::Uuid;

#[get("")]
pub async fn index(query: web::Query<PaginationParams>) -> Result<ApiResponse, ApiResponse> {
    get_all_tenants(&query).await
}

#[get("/{pid}")]
pub async fn show(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let tenant_id = path.into_inner();
    let stmt = main::entities::tenants::Entity::find_by_pid(tenant_id);

    get_tenant_by_id(stmt, &app_state, tenant_id).await
}
