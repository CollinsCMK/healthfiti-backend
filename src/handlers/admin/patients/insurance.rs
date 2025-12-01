use actix_multipart::Multipart;
use actix_web::{HttpRequest, web};
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{ColumnTrait, EntityTrait, QueryFilter},
    },
    handlers::services::patient_insurance::{
        create_patient_insurance, delete_permanently_patient_insurance, destroy_patient_insurance,
        edit_patient_insurance, fetch_patient_insurance, fetch_patient_insurances,
        restore_patient_insurance, set_primary_patient_insurance,
    },
    utils::{
        api_response::ApiResponse, app_state::AppState, pagination::PaginationParams,
        permission::has_permission,
    },
};

pub async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let mut stmt = main::entities::patients::Entity::find()
        .find_also_related(main::entities::patient_insurance::Entity);

    if !has_permission("view_archived_patient_insurances", &req).await? {
        stmt = stmt
            .filter(main::entities::patient_insurance::Column::DeletedAt.is_null())
            .filter(main::entities::patients::Column::DeletedAt.is_null());
    }

    fetch_patient_insurances(stmt, &app_state, &query).await
}

pub async fn show(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_id = path.into_inner();

    let mut stmt = main::entities::patient_insurance::Entity::find_by_pid(insurance_id)
        .find_also_related(main::entities::patients::Entity);

    if !has_permission("view_archived_patient_insurances", &req).await? {
        stmt = stmt
            .filter(main::entities::patient_insurance::Column::DeletedAt.is_null())
            .filter(main::entities::patients::Column::DeletedAt.is_null());
    }

    fetch_patient_insurance(stmt, &app_state).await
}

pub async fn create(
    payload: Multipart,
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    create_patient_insurance(payload, &app_state, req, true, None).await
}

pub async fn edit(
    payload: Multipart,
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_id = path.into_inner();

    edit_patient_insurance(payload, &app_state, req, false, None, insurance_id).await
}

pub async fn set_primary(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_id = path.into_inner();

    set_primary_patient_insurance(&app_state, insurance_id).await
}

pub async fn destroy(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_id = path.into_inner();

    destroy_patient_insurance(&app_state, insurance_id).await
}

pub async fn restore(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_id = path.into_inner();

    restore_patient_insurance(&app_state, insurance_id).await
}

pub async fn delete_permanently(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_id = path.into_inner();

    delete_permanently_patient_insurance(&app_state, insurance_id).await
}
