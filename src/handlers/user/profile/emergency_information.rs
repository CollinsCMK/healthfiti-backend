use std::collections::HashMap;

use actix_web::{HttpRequest, get, post, web};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    db::main::{self, ActiveModelTrait, ColumnTrait, QueryFilter, Set},
    utils::{
        api_response::ApiResponse, app_state::AppState, jwt::get_logged_in_user_claims,
        validator_error::ValidationError,
    },
};

#[get("/emergency-information")]
async fn get_emergency_information(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;

    let emergency_info = main::entities::patients::Entity::find_by_sso_user_id(claims.sub)
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
                    "message": "Emergency information not found"
                }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "emergency_information": emergency_info.emergency_contact,
            "message": "User emergency information fetched successfully"
        }),
    ))
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PhoneData {
    #[serde(default)]
    pub country_code: String,
    #[serde(default)]
    pub phone_number: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EmergencyContactData {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub relationship: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: PhoneData,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PreferencesData {
    #[serde(default)]
    allow_sms_notifications: bool,
    #[serde(default)]
    allow_email_notifications: bool,
    #[serde(default)]
    share_medical_info: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmergencyInfo {
    #[serde(default)]
    pub primary_contact: EmergencyContactData,
    #[serde(default)]
    pub secondary_contact: Option<EmergencyContactData>,
    #[serde(default)]
    pub support_preferences: PreferencesData,
    #[serde(default)]
    pub additional_notes: Option<String>,
}

impl EmergencyInfo {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.primary_contact.name.trim().is_empty() {
            errors.insert("name".to_string(), "Full name is required".to_string());
        }

        if self.primary_contact.relationship.trim().is_empty() {
            errors.insert(
                "relationship".to_string(),
                "Relationship is required".to_string(),
            );
        }

        if self.primary_contact.email.trim().is_empty() {
            errors.insert("email".to_string(), "Email is required".to_string());
        }

        if self.primary_contact.phone.country_code.trim().is_empty() {
            errors.insert(
                "country_code".to_string(),
                "Country code is required".to_string(),
            );
        }

        if self.primary_contact.phone.phone_number.trim().is_empty() {
            errors.insert(
                "phone_number".to_string(),
                "phone number is required".to_string(),
            );
        }

        if let Some(contact) = &self.secondary_contact {
            let all_fields_filled = !contact.name.trim().is_empty()
                && !contact.relationship.trim().is_empty()
                && !contact.email.trim().is_empty()
                && !contact.phone.country_code.trim().is_empty()
                && !contact.phone.phone_number.trim().is_empty();

            let any_field_filled = !contact.name.trim().is_empty()
                || !contact.relationship.trim().is_empty()
                || !contact.email.trim().is_empty()
                || !contact.phone.country_code.trim().is_empty()
                || !contact.phone.phone_number.trim().is_empty();

            if any_field_filled && !all_fields_filled {
                errors.insert(
                    "secondary_contact_emergency_contact".to_string(),
                    "Please fill in all secondary contact fields if you provide any information."
                        .to_string(),
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

#[post("/emergency-information/upsert")]
pub async fn upsert(
    data: web::Json<EmergencyInfo>,
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
                    "message": "User emergency information not found"
                }),
            )
        })?;

    let json_data = json!({
        "primary_contact_emergency_contact": {
            "name": data.primary_contact.name,
            "relationship": data.primary_contact.relationship,
            "email": data.primary_contact.email,
            "phone": {
                "country_code": data.primary_contact.phone.country_code,
                "phone_number": data.primary_contact.phone.phone_number,
            },
        },
        "secondary_contact_emergency_contact": {
            "name": data.secondary_contact.clone().unwrap().name,
            "relationship": data.secondary_contact.clone().unwrap().relationship,
            "email": data.secondary_contact.clone().unwrap().email,
            "phone": {
                "country_code": data.secondary_contact.clone().unwrap().phone.country_code,
                "phone_number": data.secondary_contact.clone().unwrap().phone.phone_number,
            },
        },
        "support_preferences": {
            "allow_sms_notifications": data.support_preferences.allow_sms_notifications,
            "allow_email_notifications": data.support_preferences.allow_email_notifications,
            "share_medical_info": data.support_preferences.share_medical_info,
        },
        "additional_notes": data.additional_notes,
    });
    let mut changed = false;
    let mut update_model: main::entities::patients::ActiveModel = patient.to_owned().into();

    if patient.emergency_contact != Some(json_data.clone()) {
        update_model.emergency_contact = Set(Some(json_data));
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
                "message": "Failed to update patient emergency information. Please try again later."
            }))
        })?;
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": "Patient emergency information updated successfully",
        }),
    ))
}
