use std::collections::HashMap;

use actix_multipart::Multipart;
use actix_web::{HttpRequest, get, post, web};
use chrono::{NaiveDate, Utc};
use futures::StreamExt;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{self, ActiveModelTrait, ColumnTrait, QueryFilter, Set},
    handlers::{
        auth::phone_verification::SuccessResponse,
        shared::profile::{get_profile_data, get_user_role_ids},
    },
    utils::{
        api_response::ApiResponse,
        app_state::AppState,
        http_client::ApiClient,
        jwt::get_logged_in_user_claims,
        multipart::{
            field_to_byte, field_to_date, field_to_string, get_presigned_url, upload_file,
        },
        validator_error::ValidationError,
    },
};

#[get("/personal-information")]
async fn get_personal_information(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let profile = get_profile_data(&req).await?;

    let claims = get_logged_in_user_claims(&req)?;

    let personal_info = main::entities::patients::Entity::find_by_sso_user_id(claims.sub)
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
                    "message": "Personal information not found"
                }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "personal_information": {
                "profile_picture": personal_info.photo_url,
                "first_name": profile.first_name,
                "middle_name": personal_info.middle_name,
                "last_name": profile.last_name,
                "username": profile.username,
                "email": profile.email,
                "phone": {
                    "country_code": profile.country_code,
                    "phone_number": profile.phone_number,
                },
                "is_email_verified": profile.is_email_verified,
                "is_phone_verified": profile.is_phone_verified,
                "last_login": profile.last_login,
                "is_2fa_enabled": profile.is_enabled,
                "is_secret_verified": profile.is_secret_verified,
                "method": profile.method,
                "roles": profile.roles,
                "dob": personal_info.dob,
                "gender": personal_info.gender,
                "national_id": personal_info.national_id,
                "passport_number": personal_info.passport_number,
                "address": personal_info.address,
                "city": personal_info.city,
                "county": personal_info.county,
                "country": personal_info.country,
                "created_at": profile.created_at,
            },
            "message": "User personal information fetched successfully"
        }),
    ))
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PersonalInfo {
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub middle_name: Option<String>,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub photo_url: Option<String>,
    #[serde(default)]
    pub dob: NaiveDate,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub national_id: Option<String>,
    #[serde(default)]
    pub passport_number: Option<String>,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub country_code: String,
    #[serde(default)]
    pub phone_number: String,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub county: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
}

impl PersonalInfo {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.first_name.trim().is_empty() {
            errors.insert(
                "first_name".to_string(),
                "First name is required".to_string(),
            );
        }

        if self.last_name.trim().is_empty() {
            errors.insert("last_name".to_string(), "Last name is required".to_string());
        }

        if self.username.trim().is_empty() {
            errors.insert("username".to_string(), "Username is required".to_string());
        }

        if let Some(gender) = &self.gender {
            let gender_lower = gender.to_lowercase();
            if !["male", "female", "other"].contains(&gender_lower.as_str()) {
                errors.insert(
                    "gender".into(),
                    "Gender must be one of: male, female, other.".into(),
                );
            }
        } else {
            errors.insert("gender".into(), "Gender is required.".into());
        }

        if self.email.trim().is_empty() {
            errors.insert("email".to_string(), "Email is required".to_string());
        }

        if self.country_code.trim().is_empty() {
            errors.insert(
                "country_code".to_string(),
                "Country code is required".to_string(),
            );
        }

        if self.phone_number.trim().is_empty() {
            errors.insert(
                "phone_number".to_string(),
                "Phone number is required".to_string(),
            );
        }

        if self.dob == chrono::NaiveDate::from_ymd_opt(0, 1, 1).unwrap_or_default() {
            errors.insert("dob".to_string(), "Date of birth is required".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }

    pub fn get_gender(&self) -> main::entities::sea_orm_active_enums::Gender {
        match self.gender.as_ref().map(|s| s.to_lowercase()).as_deref() {
            Some("male") => main::entities::sea_orm_active_enums::Gender::Male,
            Some("female") => main::entities::sea_orm_active_enums::Gender::Female,
            Some("other") => main::entities::sea_orm_active_enums::Gender::Other,
            _ => main::entities::sea_orm_active_enums::Gender::Other, // default fallback
        }
    }
}

#[post("/personal-information/upsert")]
pub async fn upsert(
    mut payload: Multipart,
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let claims = get_logged_in_user_claims(&req)?;
    let mut data = PersonalInfo::default();

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
            "first_name" => data.first_name = field_to_string(&mut field).await?,
            "last_name" => data.last_name = field_to_string(&mut field).await?,
            "middle_name" => data.middle_name = Some(field_to_string(&mut field).await?),
            "photo_url" => {
                let file_data = field_to_byte(&mut field).await?;
                if !file_data.is_empty() {
                    let unique_filename = format!("profile/{}-{}", Uuid::new_v4(), filename);

                    upload_file(
                        &req,
                        &app_state,
                        &unique_filename,
                        file_data.clone(),
                        &content_type,
                    )
                    .await?;

                    let url = get_presigned_url(&app_state, &unique_filename, 3600).await?;
                    data.photo_url = Some(url.clone());
                }
            }
            "dob" => data.dob = field_to_date(&mut field).await?,
            "gender" => data.gender = Some(field_to_string(&mut field).await?),
            "national_id" => data.national_id = Some(field_to_string(&mut field).await?),
            "passport_number" => data.passport_number = Some(field_to_string(&mut field).await?),
            "email" => data.email = field_to_string(&mut field).await?,
            "country_code" => data.country_code = field_to_string(&mut field).await?,
            "phone_number" => data.phone_number = field_to_string(&mut field).await?,
            "address" => data.address = Some(field_to_string(&mut field).await?),
            "city" => data.city = Some(field_to_string(&mut field).await?),
            "county" => data.county = Some(field_to_string(&mut field).await?),
            "country" => data.country = Some(field_to_string(&mut field).await?),
            _ => {}
        }
    }

    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    let role_ids = get_user_role_ids(&req).await?;

    let endpoint = format!("users/edit{}", claims.sub);

    let json_data = json!({
        "first_name": data.first_name,
        "last_name": data.last_name,
        "username": data.username,
        "email": data.email,
        "country_code": data.country_code,
        "phone_number": data.phone_number,
        "role_ids": role_ids,
    });

    let api = ApiClient::new();
    let _patient_sso: SuccessResponse = api
        .call(&endpoint, &req, Some(&json_data), Method::PUT)
        .await
        .map_err(|err| {
            log::error!("Edit user API error: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to edit user. Please try again."
                }),
            )
        })?;

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
        })?;

    if let Some(p) = patient {
        let mut changed = false;
        let mut update_model: main::entities::patients::ActiveModel = p.to_owned().into();

        if p.first_name.as_deref().unwrap_or("") != data.first_name.trim() {
            update_model.first_name = Set(Some(data.first_name.trim().to_string()));
            changed = true;
        }
        if p.last_name.as_deref().unwrap_or("") != data.last_name.trim() {
            update_model.last_name = Set(Some(data.last_name.trim().to_string()));
            changed = true;
        }
        if p.middle_name.as_deref() != data.middle_name.as_deref() {
            update_model.middle_name = Set(data.middle_name.clone());
            changed = true;
        }
        if p.photo_url.as_deref() != data.photo_url.as_deref() {
            update_model.photo_url = Set(data.photo_url.clone());
            changed = true;
        }
        if p.dob != Some(data.dob) {
            update_model.dob = Set(Some(data.dob));
            changed = true;
        }
        if p.gender != Some(data.get_gender()) {
            update_model.gender = Set(Some(data.get_gender()));
            changed = true;
        }
        if p.national_id.as_deref() != data.national_id.as_deref() {
            update_model.national_id = Set(data.national_id.clone());
            changed = true;
        }
        if p.passport_number.as_deref() != data.passport_number.as_deref() {
            update_model.passport_number = Set(data.passport_number.clone());
            changed = true;
        }
        if p.email.as_deref().unwrap_or("") != data.email.trim() {
            update_model.email = Set(Some(data.email.trim().to_string()));
            changed = true;
        }
        if p.country_code.as_deref().unwrap_or("") != data.country_code.trim() {
            update_model.country_code = Set(Some(data.country_code.trim().to_string()));
            changed = true;
        }
        if p.phone_number.as_deref().unwrap_or("") != data.phone_number.trim() {
            update_model.phone_number = Set(Some(data.phone_number.trim().to_string()));
            changed = true;
        }
        if p.address.as_deref() != data.address.as_deref() {
            update_model.address = Set(data.address.clone());
            changed = true;
        }
        if p.city.as_deref() != data.city.as_deref() {
            update_model.city = Set(data.city.clone());
            changed = true;
        }
        if p.county.as_deref() != data.county.as_deref() {
            update_model.county = Set(data.county.clone());
            changed = true;
        }
        if p.country.as_deref() != data.country.as_deref() {
            update_model.country = Set(data.country.clone());
            changed = true;
        }

        if changed {
            update_model.updated_at = Set(Utc::now().naive_utc());
            update_model
            .update(&app_state.main_db)
            .await
            .map_err(|err| {
                log::error!("Failed to update patient {}: {:?}", p.id, err);
                ApiResponse::new(500, serde_json::json!({
                    "message": "Failed to update patient personal information. Please try again later."
                }))
            })?;
        }
    }

    main::entities::patients::ActiveModel {
        sso_user_id: Set(Some(claims.sub)),
        first_name: Set(Some(data.first_name.trim().to_string())),
        last_name: Set(Some(data.last_name.trim().to_string())),
        middle_name: Set(data.middle_name.clone()),
        photo_url: Set(data.photo_url.clone()),
        dob: Set(Some(data.dob)),
        gender: Set(Some(data.get_gender())),
        national_id: Set(data.national_id.clone()),
        passport_number: Set(data.passport_number.clone()),
        email: Set(Some(data.email.trim().to_string())),
        country_code: Set(Some(data.country_code.trim().to_string())),
        phone_number: Set(Some(data.phone_number.trim().to_string())),
        address: Set(data.address.clone()),
        city: Set(data.city.clone()),
        county: Set(data.county.clone()),
        country: Set(data.country.clone()),
        ..Default::default()
    }
    .insert(&app_state.main_db)
    .await
    .map_err(|err| {
        log::error!("Failed to insert patient: {:?}", err);
        ApiResponse::new(
            500,
            serde_json::json!({
                "message": "Failed to create patient. Please try again later."
            }),
        )
    })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": "Patient personal information upserted successfully",
        }),
    ))
}
