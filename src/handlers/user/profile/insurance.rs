use std::collections::HashMap;

use actix_multipart::Multipart;
use actix_web::{HttpRequest, delete, get, patch, post, put, web};
use chrono::{NaiveDate, Utc};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        migrations::{
            Condition,
            sea_orm::{
                ActiveModelTrait, ColumnTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
            },
        },
    },
    utils::{
        api_response::ApiResponse,
        app_state::AppState,
        jwt::{get_logged_in_user_claims, get_patient_id},
        multipart::{field_to_byte, field_to_date, field_to_string, upload_file},
        pagination::PaginationParams,
        validator_error::ValidationError,
    },
};

#[get("")]
async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;

    let fetch_all = query.all.unwrap_or(false);

    let mut stmt = main::entities::patients::Entity::find_by_sso_user_id(claims.sub)
        .find_also_related(main::entities::patient_insurance::Entity);

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
            json!({
                "insurances": model,
                "success": "Patient insurance fetched successfully"
            }),
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
    let has_prev = page > 1;
    let has_next = page < total_pages;

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
            "has_prev": has_prev,
            "has_next": has_next,
            "message": "Patient insurance fetched successfully",
        }),
    ))
}

#[get("{pid}")]
async fn show(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_id = path.into_inner();
    let claims = get_logged_in_user_claims(&req)?;

    let (insurance, patient) = main::entities::patient_insurance::Entity::find_by_pid(insurance_id)
        .find_also_related(main::entities::patients::Entity)
        .filter(main::entities::patients::Column::SsoUserId.eq(claims.sub))
        .filter(main::entities::patient_insurance::Column::DeletedAt.is_null())
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

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PatientInsuranceData {
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
    pub fn validate(&self) -> Result<(), ValidationError> {
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
    app_state: &web::Data<AppState>,
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

    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    Ok(data)
}

#[post("/create")]
async fn create(
    payload: Multipart,
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let patient_id = get_patient_id(&req, &app_state).await?;
    let data = multipart_data(payload, &req, &app_state).await?;

    main::entities::patient_insurance::ActiveModel {
        patient_id: Set(patient_id),
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

#[put("/{pid}")]
async fn edit(
    payload: Multipart,
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let patient_id = get_patient_id(&req, &app_state).await?;
    let data = multipart_data(payload, &req, &app_state).await?;
    let insurance_id = path.into_inner();

    let insurance_model = main::entities::patient_insurance::Entity::find_by_pid(insurance_id)
        .filter(main::entities::patient_insurance::Column::PatientId.eq(patient_id))
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

#[patch("/{pid}/primary")]
async fn set_primary(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let patient_id = get_patient_id(&req, &app_state).await?;
    let insurance_id = path.into_inner();

    let insurance_model = main::entities::patient_insurance::Entity::find_by_pid(insurance_id)
        .filter(main::entities::patient_insurance::Column::PatientId.eq(patient_id))
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

#[delete("{pid}")]
async fn destroy(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let insurance_id = path.into_inner();
    let patient_id = get_patient_id(&req, &app_state).await?;

    let insurance_model = main::entities::patient_insurance::Entity::find_by_pid(insurance_id)
        .filter(main::entities::patient_insurance::Column::PatientId.eq(patient_id))
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
