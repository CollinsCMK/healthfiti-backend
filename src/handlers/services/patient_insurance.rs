use std::collections::HashMap;

use actix_multipart::Multipart;
use actix_web::{HttpRequest, web};
use chrono::{NaiveDate, Utc};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{
        self, entities,
        migrations::sea_orm::{
            ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter,
            QueryOrder, SelectTwo, Set,
        },
    },
    utils::{
        api_response::ApiResponse,
        app_state::AppState,
        multipart::{field_to_byte, field_to_date, field_to_i32, field_to_string, upload_file},
        pagination::PaginationParams,
        validator_error::ValidationError,
    },
};

pub async fn fetch_patient_insurances(
    stmt: SelectTwo<entities::prelude::Patients, entities::prelude::PatientInsurance>,
    app_state: &AppState,
    query: &PaginationParams,
) -> Result<ApiResponse, ApiResponse> {
    let fetch_all = query.all.unwrap_or(false);

    let mut stmt = stmt;
    if let Some(term) = &query.search {
        use main::migrations::{Expr, extension::postgres::PgExpr};
        let like = format!("%{}%", term);
        stmt = stmt.filter(
            Condition::any()
                .add(
                    Expr::col((
                        main::entities::patient_insurance::Entity,
                        main::entities::patient_insurance::Column::Provider,
                    ))
                    .ilike(like.clone()),
                )
                .add(
                    Expr::col((
                        main::entities::patient_insurance::Entity,
                        main::entities::patient_insurance::Column::PolicyNumber,
                    ))
                    .ilike(like.clone()),
                )
                .add(
                    Expr::col((
                        main::entities::patient_insurance::Entity,
                        main::entities::patient_insurance::Column::GroupNumber,
                    ))
                    .ilike(like.clone()),
                ),
        );
    }

    if fetch_all {
        let model = stmt
            .order_by_asc(main::entities::patient_insurance::Column::CreatedAt)
            .order_by_asc(main::entities::patient_insurance::Column::Id)
            .all(&app_state.main_db)
            .await
            .map_err(|err| ApiResponse::new(500, json!({"message": err.to_string()})))?
            .into_iter()
            .map(|(_, ins)| {
                if let Some(ins) = ins {
                    json!({
                        "pid": ins.pid,
                        "provider": ins.provider,
                        "policy_number": ins.policy_number,
                        "group_number": ins.group_number,
                        "plan_type": ins.plan_type,
                        "coverage_start_date": ins.coverage_start_date,
                        "coverage_end_date": ins.coverage_end_date,
                        "is_primary": ins.is_primary,
                    })
                } else {
                    json!(null)
                }
            })
            .collect::<Vec<_>>();

        return Ok(ApiResponse::new(
            200,
            json!({ "insurances": model, "success": "Patient insurance fetched successfully" }),
        ));
    }

    let page = query.page.unwrap_or(1).min(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let paginator = stmt.paginate(&app_state.main_db, limit);
    let total_items = paginator
        .num_items()
        .await
        .map_err(|err| ApiResponse::new(500, json!({"message": err.to_string()})))?;
    let total_pages = (total_items as f64 / limit as f64).ceil() as u64;

    let model = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .map_err(|err| ApiResponse::new(500, json!({"message": err.to_string()})))?
        .into_iter()
        .map(|(patient, ins)| {
            if let Some(ins) = ins {
                json!({
                    "pid": ins.pid,
                    "provider": ins.provider,
                    "policy_number": ins.policy_number,
                    "group_number": ins.group_number,
                    "plan_type": ins.plan_type,
                    "coverage_start_date": ins.coverage_start_date,
                    "coverage_end_date": ins.coverage_end_date,
                    "is_primary": ins.is_primary,
                    "patient": {
                        "pid": patient.pid,
                        "name": format!("{:?} {:?}", patient.first_name, patient.last_name),
                    },
                    "created_at": ins.created_at,
                })
            } else {
                json!(null)
            }
        })
        .collect::<Vec<_>>();

    Ok(ApiResponse::new(
        200,
        json!({
            "insurances": model,
            "page": page,
            "total_pages": total_pages,
            "total_items": total_items,
            "has_prev": page > 1,
            "has_next": page < total_pages,
            "message": "Patient insurance fetched successfully"
        }),
    ))
}

pub async fn fetch_patient_insurance(
    stmt: SelectTwo<entities::prelude::PatientInsurance, entities::prelude::Patients>,
    app_state: &AppState,
) -> Result<ApiResponse, ApiResponse> {
    let stmt = stmt;

    let (insurance, patient) = stmt
        .one(&app_state.main_db)
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?
        .ok_or_else(|| {
            ApiResponse::new(404, json!({ "message": "Patient insurance not found" }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "insurance": {
                "pid": insurance.pid,
                "provider": insurance.provider,
                "policy_number": insurance.policy_number,
                "group_number": insurance.group_number,
                "plan_type": insurance.plan_type,
                "coverage_start_date": insurance.coverage_start_date,
                "coverage_end_date": insurance.coverage_end_date,
                "is_primary": insurance.is_primary,
                "patient": {
                    "pid": patient.as_ref().map(|p| p.pid),
                    "name": patient.as_ref().map(|p| {
                        let first = p.first_name.as_deref().unwrap_or("");
                        let last = p.last_name.as_deref().unwrap_or("");
                        format!("{} {}", first, last).trim().to_string()
                    })
                },
                "created_at": insurance.created_at,
                "updated_at": insurance.updated_at,
                "deleted_at": insurance.deleted_at,
            },
            "message": "Patient insurance fetched successfully",
        }),
    ))
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct PatientInsuranceData {
    pub patient_id: Option<i32>,
    pub provider: String,
    pub policy_number: String,
    pub group_number: Option<String>,
    pub plan_type: String,
    pub coverage_start_date: NaiveDate,
    pub coverage_end_date: NaiveDate,
    pub is_primary: bool,
    pub insurance_card_front: String,
    pub insurance_card_back: String,
}

impl PatientInsuranceData {
    pub fn validate(&self, is_admin: bool) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.provider.trim().is_empty() {
            errors.insert("provider".into(), "Provider is required.".into());
        }

        if self.policy_number.trim().is_empty() {
            errors.insert("policy_number".into(), "Policy number is required.".into());
        }

        if self.plan_type.trim().is_empty() {
            errors.insert("plan_type".into(), "Plan type is required.".into());
        }

        if self.coverage_start_date < Utc::now().date_naive() {
            errors.insert(
                "coverage_start_date".into(),
                "Coverage start date cannot be in the past.".into(),
            );
        }

        if self.coverage_end_date < Utc::now().date_naive() {
            errors.insert(
                "coverage_end_date".into(),
                "Coverage end date cannot be in the past.".into(),
            );
        }

        if self.coverage_end_date < self.coverage_start_date {
            errors.insert(
                "coverage_dates".into(),
                "Coverage end date cannot be before start date.".into(),
            );
        }

        if self.insurance_card_front.trim().is_empty() {
            errors.insert(
                "insurance_card_front".into(),
                "Insurance card front is required.".into(),
            );
        }

        if self.insurance_card_back.trim().is_empty() {
            errors.insert(
                "insurance_card_back".into(),
                "Insurance card back is required.".into(),
            );
        }

        if is_admin && self.patient_id.is_none() {
            errors.insert(
                "patient_id".into(),
                "Patient ID is required for admin.".into(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

async fn multipart_data(
    mut payload: Multipart,
    req: &HttpRequest,
    app_state: &AppState,
    is_admin: bool,
    patient_id: Option<i32>,
) -> Result<PatientInsuranceData, ApiResponse> {
    let mut data = PatientInsuranceData::default();

    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition().unwrap();
        let name = content_disposition.get_name().unwrap_or("");
        let filename = content_disposition
            .get_filename()
            .map(|f| f.to_string())
            .unwrap_or_else(|| format!("{}.bin", Uuid::new_v4()));
        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        match name {
            "patient_id" => {
                data.patient_id = if let Some(id) = patient_id {
                    Some(id)
                } else {
                    Some(field_to_i32(&mut field).await?)
                }
            }
            "provider" => data.provider = field_to_string(&mut field).await?,
            "policy_number" => data.policy_number = field_to_string(&mut field).await?,
            "group_number" => data.group_number = Some(field_to_string(&mut field).await?),
            "plan_type" => data.plan_type = field_to_string(&mut field).await?,
            "coverage_start_date" => data.coverage_start_date = field_to_date(&mut field).await?,
            "coverage_end_date" => data.coverage_end_date = field_to_date(&mut field).await?,
            "insurance_card_front" => {
                let file_data = field_to_byte(&mut field).await?;
                if !file_data.is_empty() {
                    let unique_filename =
                        format!("insurance_card_front/{}-{}", Uuid::new_v4(), filename);

                    let full_s3_key = upload_file(
                        &req,
                        &app_state,
                        &unique_filename,
                        file_data.clone(),
                        &content_type,
                    )
                    .await?;

                    data.insurance_card_front = full_s3_key;
                }
            }
            "insurance_card_back" => {
                let file_data = field_to_byte(&mut field).await?;
                if !file_data.is_empty() {
                    let unique_filename =
                        format!("insurance_card_back/{}-{}", Uuid::new_v4(), filename);

                    let full_s3_key = upload_file(
                        &req,
                        &app_state,
                        &unique_filename,
                        file_data.clone(),
                        &content_type,
                    )
                    .await?;

                    data.insurance_card_back = full_s3_key;
                }
            }
            _ => {}
        }
    }

    if let Err(err) = data.validate(is_admin) {
        return Err(ApiResponse::new(400, json!(err)));
    }

    Ok(data)
}

pub async fn create_patient_insurance(
    payload: Multipart,
    app_state: &AppState,
    req: HttpRequest,
    is_admin: bool,
    patient_id: Option<i32>,
) -> Result<ApiResponse, ApiResponse> {
    let data = multipart_data(payload, &req, &app_state, is_admin, patient_id).await?;

    main::entities::patient_insurance::ActiveModel {
        patient_id: Set(data.patient_id.expect("Patient ID")),
        provider: Set(data.provider.trim().to_string()),
        policy_number: Set(data.policy_number.trim().to_string()),
        group_number: Set(data.group_number.clone()),
        plan_type: Set(Some(data.plan_type.clone())),
        coverage_start_date: Set(Some(data.coverage_start_date)),
        coverage_end_date: Set(Some(data.coverage_end_date)),
        is_primary: Set(data.is_primary),
        insurance_card_front: Set(Some(data.insurance_card_front)),
        insurance_card_back: Set(Some(data.insurance_card_back)),
        ..Default::default()
    }
    .insert(&app_state.main_db)
    .await
    .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    Ok(ApiResponse::new(
        201,
        json!({ "message": "Patient insurance created successfully" }),
    ))
}

pub async fn edit_patient_insurance(
    payload: Multipart,
    app_state: &web::Data<AppState>,
    req: HttpRequest,
    is_admin: bool,
    patient_id: Option<i32>,
    insurance_id: Uuid,
) -> Result<ApiResponse, ApiResponse> {
    let data = multipart_data(payload, &req, &app_state, is_admin, patient_id).await?;

    let insurance_model = main::entities::patient_insurance::Entity::find_by_pid(insurance_id)
        .filter(
            main::entities::patient_insurance::Column::PatientId
                .eq(data.patient_id.expect("Patient ID")),
        )
        .filter(main::entities::patient_insurance::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch patient insurance: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to fetch patient insurance" }),
            )
        })?
        .ok_or_else(|| {
            log::error!("Patient insurance not found for id: {}", insurance_id);
            ApiResponse::new(404, json!({ "message": "Patient insurance not found" }))
        })?;

    let mut update_model: main::entities::patient_insurance::ActiveModel =
        insurance_model.to_owned().into();
    let mut changed = false;

    if insurance_model.provider.clone() != data.provider.trim() {
        update_model.provider = Set(data.provider.trim().to_string());
        changed = true;
    }
    if insurance_model.policy_number.clone() != data.policy_number.trim() {
        update_model.policy_number = Set(data.policy_number.trim().to_string());
        changed = true;
    }
    if insurance_model.group_number != data.group_number {
        update_model.group_number = Set(data.group_number.clone());
        changed = true;
    }
    if insurance_model.plan_type.as_deref() != Some(data.plan_type.as_str()) {
        update_model.plan_type = Set(Some(data.plan_type.clone()));
        changed = true;
    }
    if insurance_model.coverage_start_date != Some(data.coverage_start_date) {
        update_model.coverage_start_date = Set(Some(data.coverage_start_date));
        changed = true;
    }
    if insurance_model.coverage_end_date != Some(data.coverage_end_date) {
        update_model.coverage_end_date = Set(Some(data.coverage_end_date));
        changed = true;
    }
    if insurance_model.is_primary != data.is_primary {
        update_model.is_primary = Set(data.is_primary);
        changed = true;
    }

    if !changed {
        return Err(ApiResponse::new(
            400,
            json!({ "message": "No updates were made because the data is unchanged." }),
        ));
    }

    update_model.updated_at = Set(chrono::Utc::now().naive_utc());
    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update patient insurance: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to update patient insurance" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": "Patient insurance updated successfully" }),
    ))
}

pub async fn set_primary_patient_insurance(
    app_state: &web::Data<AppState>,
    insurance_id: Uuid,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_model = main::entities::patient_insurance::Entity::find_by_pid(insurance_id)
        .filter(main::entities::patient_insurance::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch patient insurance: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to fetch patient insurance" }),
            )
        })?
        .ok_or_else(|| {
            log::error!("Patient insurance not found for id: {}", insurance_id);
            ApiResponse::new(404, json!({ "message": "Patient insurance not found" }))
        })?;

    let was_primary = insurance_model.is_primary;
    let new_status = !was_primary;

    let mut update_insurance_model: main::entities::patient_insurance::ActiveModel =
        insurance_model.to_owned().into();
    update_insurance_model.is_primary = Set(new_status);
    update_insurance_model.updated_at = Set(Utc::now().naive_utc());
    update_insurance_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!(
                "Failed to update primary status of patient insurance: {}",
                err
            );
            ApiResponse::new(
                500,
                json!({ "message": "Failed to update patient insurance status" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": "Patient insurance primary status updated successfully",
        }),
    ))
}

pub async fn destroy_patient_insurance(
    app_state: &web::Data<AppState>,
    insurance_id: Uuid,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_model = main::entities::patient_insurance::Entity::find_by_pid(insurance_id)
        .filter(main::entities::patient_insurance::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch patient insurance: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to fetch patient insurance" }),
            )
        })?
        .ok_or_else(|| {
            log::error!("Patient insurance not found for id: {}", insurance_id);
            ApiResponse::new(404, json!({ "message": "Patient insurance not found" }))
        })?;

    let mut update_model: main::entities::patient_insurance::ActiveModel =
        insurance_model.to_owned().into();
    update_model.deleted_at = Set(Some(chrono::Utc::now().naive_utc()));
    update_model.updated_at = Set(chrono::Utc::now().naive_utc());

    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to delete patient insurance: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to delete patient insurance" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": "Patient insurance deleted successfully" }),
    ))
}

pub async fn restore_patient_insurance(
    app_state: &web::Data<AppState>,
    insurance_id: Uuid,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_model = main::entities::patient_insurance::Entity::find_by_pid(insurance_id)
        .filter(main::entities::patient_insurance::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch patient insurance: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to fetch patient insurance" }),
            )
        })?
        .ok_or_else(|| {
            log::error!("Patient insurance not found for id: {}", insurance_id);
            ApiResponse::new(404, json!({ "message": "Patient insurance not found" }))
        })?;

    let mut update_model: main::entities::patient_insurance::ActiveModel =
        insurance_model.to_owned().into();
    update_model.deleted_at = Set(None);
    update_model.updated_at = Set(chrono::Utc::now().naive_utc());

    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to delete patient insurance: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to delete patient insurance" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": "Patient insurance deleted successfully" }),
    ))
}

pub async fn delete_permanently_patient_insurance(
    app_state: &web::Data<AppState>,
    insurance_id: Uuid,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_data = main::entities::patient_insurance::Entity::find_by_pid(insurance_id)
        .one(&app_state.main_db)
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?
        .ok_or_else(|| ApiResponse::new(404, json!({ "message": "Role not found" })))?;

    let result = main::entities::patient_insurance::Entity::delete_by_id(insurance_data.id)
        .exec(&app_state.main_db)
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    if result.rows_affected == 0 {
        return Err(ApiResponse::new(
            404,
            json!({ "message": "Role not found." }),
        ));
    }

    Ok(ApiResponse::new(
        200,
        json!({ "message": "Role permanently deleted successfully" }),
    ))
}
