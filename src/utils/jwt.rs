use actix_web::{HttpMessage, HttpRequest, web};
use jsonwebtoken::{dangerous::insecure_decode, errors::Error};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect},
    },
    utils::{api_response::ApiResponse, app_state::AppState},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub tenant_pid: Option<Uuid>,
    pub role_pid: Uuid,
    pub role_name: String,
    pub application_pid: Option<Uuid>,
    pub device_id: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    pub iss: String,
    pub token_type: String,
}

pub fn decode_token(token: &str) -> Result<Claims, Error> {
    let token_data = insecure_decode::<Claims>(token)?;
    Ok(token_data.claims)
}

pub fn get_logged_in_user_claims(req: &HttpRequest) -> Result<Claims, ApiResponse> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| ApiResponse::new(401, json!({ "message": "Unauthorized" })))
}

pub fn get_logged_in_tenant_pid(req: &HttpRequest) -> Result<Uuid, ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;
    claims
        .tenant_pid
        .ok_or(ApiResponse::new(401, json!({ "message": "Unauthorized" })))
}

pub async fn get_patient_id(
    req: &HttpRequest,
    app_state: &web::Data<AppState>,
    patient_id: Option<Uuid>,
) -> Result<(i32, Uuid), ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;

    let mut stmt = main::entities::patients::Entity::find()
        .filter(main::entities::patients::Column::DeletedAt.is_null());

    if let Some(id) = patient_id {
        stmt = stmt.filter(main::entities::patients::Column::Pid.eq(id));
    } else {
        stmt = stmt.filter(main::entities::patients::Column::SsoUserId.eq(claims.sub))
    };

    let (patient_id, patient_pid) = stmt
        .select_only()
        .column(main::entities::patients::Column::Id)
        .column(main::entities::patients::Column::Pid)
        .into_tuple::<(i32, Uuid)>()
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch patient id: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to fetch patient id" }))
        })?
        .ok_or_else(|| ApiResponse::new(404, json!({ "message": "Patient not found" })))?;

    Ok((patient_id, patient_pid))
}

pub async fn get_tenant_id(
    req: &HttpRequest,
    app_state: &web::Data<AppState>,
) -> Result<(i32, Uuid, Uuid), ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;

    let mut stmt = main::entities::tenants::Entity::find()
        .filter(main::entities::tenants::Column::DeletedAt.is_null());

    if let Some(tenant_pid) = claims.tenant_pid {
        stmt = stmt.filter(main::entities::tenants::Column::Pid.eq(tenant_pid));
    } else {
        return Err(ApiResponse::new(401, json!({ "message": "Unauthorized" })));
    }

    let (tenant_id, tenant_pid, tenant_sso_id) = stmt
        .select_only()
        .column(main::entities::tenants::Column::Id)
        .column(main::entities::tenants::Column::Pid)
        .column(main::entities::tenants::Column::SsoTenantId)
        .into_tuple::<(i32, Uuid, Uuid)>()
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch tenant id: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to fetch tenant id" }))
        })?
        .ok_or_else(|| ApiResponse::new(404, json!({ "message": "Tenant not found" })))?;

    Ok((tenant_id, tenant_pid, tenant_sso_id))
}
