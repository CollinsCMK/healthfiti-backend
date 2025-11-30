use actix_multipart::Multipart;
use actix_web::{HttpRequest, delete, get, patch, post, put, web};
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{ColumnTrait, QueryFilter},
    },
    handlers::services::patient_insurance::{
        create_patient_insurance, destroy_patient_insurance, edit_patient_insurance,
        fetch_patient_insurance, fetch_patient_insurances, set_primary_patient_insurance,
    },
    utils::{
        api_response::ApiResponse,
        app_state::AppState,
        jwt::{get_logged_in_user_claims, get_patient_id},
        pagination::PaginationParams,
    },
};

#[get("")]
async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;
    let stmt = main::entities::patients::Entity::find_by_sso_user_id(claims.sub)
        .find_also_related(main::entities::patient_insurance::Entity);

    fetch_patient_insurances(stmt, &app_state, &query).await
}

#[get("{pid}")]
async fn show(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_id = path.into_inner();
    let claims = get_logged_in_user_claims(&req)?;

    let stmt = main::entities::patient_insurance::Entity::find_by_pid(insurance_id)
        .find_also_related(main::entities::patients::Entity)
        .filter(main::entities::patients::Column::SsoUserId.eq(claims.sub))
        .filter(main::entities::patient_insurance::Column::DeletedAt.is_null());

    fetch_patient_insurance(stmt, &app_state).await
}

#[post("/create")]
async fn create(
    payload: Multipart,
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let (patient_id, _) = get_patient_id(&req, &app_state, None).await?;

    create_patient_insurance(payload, &app_state, req, false, Some(patient_id)).await
}

#[put("/{pid}")]
async fn edit(
    payload: Multipart,
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let (patient_id, _) = get_patient_id(&req, &app_state, None).await?;
    let insurance_id = path.into_inner();

    edit_patient_insurance(
        payload,
        &app_state,
        req,
        false,
        Some(patient_id),
        insurance_id,
    )
    .await
}

#[patch("/{pid}/primary")]
async fn set_primary(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let (_, _) = get_patient_id(&req, &app_state, None).await?;
    let insurance_id = path.into_inner();

    set_primary_patient_insurance(&app_state, insurance_id).await
}

#[delete("{pid}")]
async fn destroy(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_id = path.into_inner();
    let (_, _) = get_patient_id(&req, &app_state, None).await?;

    destroy_patient_insurance(&app_state, insurance_id).await
}
