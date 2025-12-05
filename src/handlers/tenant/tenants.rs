use crate::{
    db::main,
    handlers::services::tenants::{TenantData, edit_tenant, get_tenant_by_id},
    utils::{api_response::ApiResponse, app_state::AppState, jwt::get_logged_in_tenant_pid},
};
use actix_web::{HttpRequest, web};

pub async fn show(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> Result<ApiResponse, ApiResponse> {
    let tenant_id = get_logged_in_tenant_pid(&req)?;

    let stmt = main::entities::tenants::Entity::find_by_pid(tenant_id);

    get_tenant_by_id(stmt, &app_state, tenant_id).await
}

pub async fn update(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    data: web::Json<TenantData>,
) -> Result<ApiResponse, ApiResponse> {
    let tenant_id = get_logged_in_tenant_pid(&req)?;

    edit_tenant(&app_state, tenant_id, &data).await
}
