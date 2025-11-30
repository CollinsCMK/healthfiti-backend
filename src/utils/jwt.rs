use actix_web::{HttpMessage, HttpRequest, web};
use jsonwebtoken::{dangerous::insecure_decode, errors::Error};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{ColumnTrait, QueryFilter, QuerySelect},
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

pub async fn get_patient_id(
    req: &HttpRequest,
    app_state: &web::Data<AppState>,
) -> Result<i32, ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;

    let patient_id = main::entities::patients::Entity::find_by_sso_user_id(claims.sub)
        .filter(main::entities::patients::Column::DeletedAt.is_null())
        .select_only()
        .column(main::entities::patients::Column::Id)
        .into_tuple::<i32>()
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch patient id: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to fetch patient id" }))
        })?
        .ok_or_else(|| ApiResponse::new(404, json!({ "message": "Patient not found" })))?;

    Ok(patient_id)
}
