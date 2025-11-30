use std::collections::HashMap;

use actix_web::{HttpRequest, get, post, web};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{ActiveModelTrait, ColumnTrait, QueryFilter, Set},
    },
    utils::{
        api_response::ApiResponse, app_state::AppState, jwt::get_logged_in_user_claims,
        validator_error::ValidationError,
    },
};

#[get("/health-information")]
async fn get_health_information(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;

    let health_info = main::entities::patients::Entity::find_by_sso_user_id(claims.sub)
        .filter(main::entities::patients::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!(
                "Failed to query patient by sso_user_id {}: {:?}",
                claims.sub,
                err
            );

            ApiResponse::new(
                500,
                json!({
                    "message": "Internal server error. Please try again later."
                }),
            )
        })?
        .ok_or_else(|| {
            ApiResponse::new(
                404,
                json!({
                    "message": "Health information not found"
                }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "health_information": {
                "blood_type": health_info.blood_type,
                "allergies": health_info.allergies,
                "medical_conditions": health_info.medical_conditions,
            },
            "message": "User health information fetched successfully"
        }),
    ))
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HealthInfo {
    #[serde(default)]
    pub blood_type: Option<String>,
    #[serde(default)]
    pub allergies: Option<Vec<String>>,
    #[serde(default)]
    pub medical_conditions: Option<Vec<String>>,
}

impl HealthInfo {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        const VALID_BLOOD_TYPES: [&str; 8] = ["A+", "A-", "B+", "B-", "AB+", "AB-", "O+", "O-"];

        if let Some(blood) = &self.blood_type {
            let normalized = blood.trim().to_uppercase();

            if !VALID_BLOOD_TYPES.contains(&normalized.as_str()) {
                errors.insert(
                    "blood_type".to_string(),
                    "Invalid blood type. Allowed: A+, A-, B+, B-, AB+, AB-, O+, O-".to_string(),
                );
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

#[post("/health-information/upsert")]
pub async fn upsert(
    data: web::Json<HealthInfo>,
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;

    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    let patient = main::entities::patients::Entity::find_by_sso_user_id(claims.sub)
        .filter(main::entities::patients::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!(
                "Failed to fetch patient for sso_user_id {}: {:?}",
                claims.sub,
                err
            );

            ApiResponse::new(
                500,
                json!({
                    "message": "Internal server error. Please try again later."
                }),
            )
        })?
        .ok_or_else(|| {
            ApiResponse::new(
                404,
                json!({
                    "message": "User health information not found"
                }),
            )
        })?;

    let mut changed = false;
    let mut update_model: main::entities::patients::ActiveModel = patient.to_owned().into();

    if patient.blood_type != data.blood_type {
        update_model.blood_type = Set(data.blood_type.clone());
        changed = true;
    }
    if patient.allergies != data.allergies {
        update_model.allergies = Set(data.allergies.clone());
        changed = true;
    }
    if patient.medical_conditions != data.medical_conditions {
        update_model.medical_conditions = Set(data.medical_conditions.clone());
        changed = true;
    }

    if changed {
        update_model.updated_at = Set(Utc::now().naive_utc());
        update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update patient {}: {:?}", patient.id, err);
            ApiResponse::new(500, serde_json::json!({
                "message": "Failed to update patient health information. Please try again later."
            }))
        })?;
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": "Patient health information updated successfully",
        }),
    ))
}
