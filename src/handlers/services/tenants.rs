use std::collections::HashMap;

use chrono::{NaiveDateTime, Utc};
use chrono_tz::Tz;
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Select, Set},
    },
    handlers::auth::phone_verification::SuccessResponse,
    utils::{api_response::ApiResponse, app_state::AppState, http_client::ApiClient, pagination::PaginationParams, validator_error::ValidationError},
};


#[derive(Serialize, Deserialize, Debug)]
pub struct TenantResponse {
    pub pid: Uuid,
    pub name: String,
    pub deleted_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginationInfo {
    pub page: u64,
    pub total_pages: u64,
    pub total_items: u64,
    pub has_prev: bool,
    pub has_next: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponseDTO<T> {
    pub data: Option<T>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationInfo>,
}

pub async fn get_all_tenants(
    query: &PaginationParams,
) -> Result<ApiResponse, ApiResponse> {
    let fetch_all = query.all.unwrap_or(false);

    let api = ApiClient::new();
    let mut endpoint = format!(
        "tenants?all={}&page={}&limit={}",
        fetch_all,
        query.page.unwrap_or(1),
        query.limit.unwrap_or(10)
    );

    if let Some(term) = &query.search {
        endpoint.push_str(&format!("&search={}", term));
    }

    let response: ApiResponseDTO<Vec<TenantResponse>> = api
        .call(&endpoint, &None, None::<&()>, Method::GET)
        .await
        .map_err(|err| {
            log::error!("Failed to update tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to update tenant" }))
        })?;

    if let Some(tenants) = response.data {
        if let Some(p) = response.pagination {
            return Ok(ApiResponse::new(
                200,
                json!({
                    "tenants": tenants,
                    "page": p.page,
                    "total_pages": p.total_pages,
                    "total_items": p.total_items,
                    "has_prev": p.has_prev,
                    "has_next": p.has_next,
                    "message": response.message,
                }),
            ));
        }

        return Ok(ApiResponse::new(
            200,
            json!({
                "tenants": tenants,
                "message": response.message,
            }),
        ));
    }

    Err(ApiResponse::new(
        500,
        json!({ "message": "Failed to fetch tenants" }),
    ))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SingleTenantResponse {
    pub pid: Uuid,
    pub name: String,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
}

pub async fn get_tenant_by_id(
    stmt: Select<main::entities::tenants::Entity>,
    app_state: &AppState,
    tenant_pid: Uuid,
) -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();
    let endpoint = format!("tenants/show/{}", tenant_pid);
    let response: ApiResponseDTO<SingleTenantResponse> = api
        .call(&endpoint, &None, None::<&()>, Method::GET)
        .await
        .map_err(|err| {
            log::error!("Failed to find tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to find tenant" }))
        })?;

    let tenant = stmt
        .filter(main::entities::tenants::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to find tenant" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Tenant not found" }),
        ))?;

    if let Some(t) = response.data {
        return Ok(ApiResponse::new(
            200,
            json!({
                "tenant": {
                    "sso_tenant_id": t.pid,
                    "name": t.name,
                    "pid": tenant.pid,
                    "slug": tenant.slug,
                    "country": tenant.country,
                    "county": tenant.county,
                    "city": tenant.city,
                    "latitude": tenant.latitude,
                    "longitude": tenant.longitude,
                    "db_url": tenant.db_url,
                    "subscription_tier": tenant.subscription_tier,
                    "subscription_status": tenant.subscription_status,
                    "trial_ends_at": tenant.trial_ends_at,
                    "subscription_started_at": tenant.subscription_started_at,
                    "subscription_ends_at": tenant.subscription_ends_at,
                    "contact_email": tenant.contact_email,
                    "contact_phone": tenant.contact_phone,
                    "timezone": tenant.timezone,
                    "currency": tenant.currency,
                    "settings": tenant.settings,
                    "onboarding_completed": tenant.onboarding_completed,
                    "status": t.status,
                    "created_at": t.created_at,
                    "updated_at": t.updated_at,
                    "deleted_at": t.deleted_at,
                },
                "message": response.message,
            }),
        ));
    }

    Err(ApiResponse::new(
        500,
        json!({ "message": response.message }),
    ))
}

#[derive(Deserialize, Debug)]
pub struct TenantCreateResponse {
    pub pid: Option<Uuid>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TenantData {
    pub name: String,
    pub status: Option<String>,
    pub country: Option<String>,
    pub county: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub db_url: String,
    pub contact_email: Option<String>,
    pub country_code: Option<String>,
    pub contact_phone: Option<String>,
    pub timezone: String,
    pub currency: String,
}

impl TenantData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.name.trim().is_empty() {
            errors.insert("name".into(), "Tenant name is required.".into());
        }

        if let Some(status) = &self.status {
            let status_lower = status.to_lowercase();
            if !["active", "suspended", "trial", "expired"].contains(&status_lower.as_str()) {
                errors.insert(
                    "status".into(),
                    "Status must be one of: active, suspended, trial, expired.".into(),
                );
            }
        } else {
            errors.insert("status".into(), "Status is required.".into());
        }

        if self.db_url.trim().is_empty() {
            errors.insert("db_url".into(), "Database URL is required.".into());
        }

        if self.timezone.trim().is_empty() {
            errors.insert("timezone".into(), "Timezone is required.".into());
        } else if self.timezone.parse::<Tz>().is_err() {
            errors.insert("timezone".into(), "Timezone is invalid.".into());
        }

        if self.currency.trim().is_empty() {
            errors.insert("currency".into(), "Currency is required.".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

pub async fn create_tenant(
    app_state: &AppState,
    data: &TenantData,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let api = ApiClient::new();

    let json_value = json!({
        "name": data.name,
        "status": data.status,
    });

    let response: TenantCreateResponse = api
        .call("tenants/create", &None, Some(&json_value), Method::POST)
        .await
        .map_err(|err| {
            log::error!("Failed to create tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to create tenant" }))
        })?;

    if let Some(pid) = response.pid {
        main::entities::tenants::ActiveModel {
            sso_tenant_id: Set(pid),
            country: Set(data.country.clone()),
            county: Set(data.county.clone()),
            city: Set(data.city.clone()),
            latitude: Set(data.latitude),
            longitude: Set(data.longitude),
            db_url: Set(data.db_url.clone()),
            contact_email: Set(data.contact_email.clone()),
            country_code: Set(data.country_code.clone()),
            contact_phone: Set(data.contact_phone.clone()),
            timezone: Set(data.timezone.clone()),
            currency: Set(data.currency.clone()),
            ..Default::default()
        }
            .insert(&app_state.main_db)
            .await
            .map_err(|err| {
                log::error!("Failed to create tenant: {}", err);
                ApiResponse::new(500, json!({ "message": "Failed to create tenant" }))
            })?;

        return Ok(ApiResponse::new(
            200,
            json!({ "message": "Tenant created successfully" }),
        ));
    }  

    Err(ApiResponse::new(
        500,
        json!({ "message": response.message }),
    ))
}

pub async fn edit_tenant(
    app_state: &AppState,
    tenant_pid: Uuid,
    data: &TenantData,
) -> Result<ApiResponse, ApiResponse> {
    let tenant = main::entities::tenants::Entity::find_by_pid(tenant_pid)
        .filter(main::entities::tenants::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to find tenant" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Tenant not found" }),
        ))?;

    let api = ApiClient::new();
    let endpoint = format!("tenants/edit/{}", tenant.sso_tenant_id);

    let json_value = json!({
        "name": data.name,
        "status": data.status,
    });

    let _response: SuccessResponse = api
        .call(&endpoint, &None, Some(&json_value), Method::POST)
        .await
        .map_err(|err| {
            log::error!("Failed to update tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to update tenant" }))
        })?;

    // if response.message {
        let mut update_model: main::entities::tenants::ActiveModel = tenant.to_owned().into();
        let mut changed = false;

        if data.country != tenant.country {
            update_model.country = Set(data.country.clone());
            changed = true;
        }

        if data.county != tenant.county {
            update_model.county = Set(data.county.clone());
            changed = true;
        }

        if data.city != tenant.city {
            update_model.city = Set(data.city.clone());
            changed = true;
        }

        if data.latitude != tenant.latitude {
            update_model.latitude = Set(data.latitude.clone());
            changed = true;
        }

        if data.longitude != tenant.longitude {
            update_model.longitude = Set(data.longitude.clone());
            changed = true;
        }

        if data.db_url != tenant.db_url {
            update_model.db_url = Set(data.db_url.clone());
            changed = true;
        }

        if data.contact_email != tenant.contact_email {
            update_model.contact_email = Set(data.contact_email.clone());
            changed = true;
        }

        if data.country_code != tenant.country_code {
            update_model.country_code = Set(data.country_code.clone());
            changed = true;
        }

        if data.contact_phone != tenant.contact_phone {
            update_model.contact_phone = Set(data.contact_phone.clone());
            changed = true;
        }

        if data.timezone != tenant.timezone {
            update_model.timezone = Set(data.timezone.clone());
            changed = true;
        }

        if data.currency != tenant.currency {
            update_model.currency = Set(data.currency.clone());
            changed = true;
        }

        if !changed {
            return Err(ApiResponse::new(
                400,
                json!({
                    "message": "No updates were made because the data is unchanged."
                }),
            ));
        }

        update_model.updated_at = Set(Utc::now().naive_utc());
        update_model.update(&app_state.main_db).await.map_err(|err| {
                log::error!("Failed to update tenant: {}", err);
                ApiResponse::new(500, json!({ "message": "Failed to update tenant" }))
            })?;

        Ok(ApiResponse::new(
            200,
            json!({ "message": "Tenant updated successfully" }),
        ))
    // }

    // Err(ApiResponse::new(
    //     500,
    //     json!({ "message": response.message }),
    // ))
}

#[derive(Deserialize, Debug)]
pub struct ActivateTenantResponse {
    pub errors: Option<HashMap<String, String>>,
    pub message: String,
}

pub async fn set_active_status_tenant(
    app_state: &AppState,
    tenant_pid: Uuid,
    status: &str,
) -> Result<ApiResponse, ApiResponse> {
    let tenant = main::entities::tenants::Entity::find_by_pid(tenant_pid)
        .filter(main::entities::tenants::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to find tenant" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Tenant not found" }),
        ))?;

    let api = ApiClient::new();
    let endpoint = format!("tenants/status/{}", tenant.sso_tenant_id);

    let json_value = json!({
        "status": status,
    });

    let response: ActivateTenantResponse = api
        .call(&endpoint, &None, Some(&json_value), Method::POST)
        .await
        .map_err(|err| {
            log::error!("Failed to update tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to update tenant" }))
        })?;

    if let Some(errors) = response.errors {
        return Err(ApiResponse::new(400, json!({ "errors": errors })));
    }

    Ok(ApiResponse::new(
        200,
        json!({ "message": response.message }),
    ))
}

pub async fn destroy_tenant(
    app_state: &AppState,
    tenant_pid: Uuid,
) -> Result<ApiResponse, ApiResponse> {
    let tenant = main::entities::tenants::Entity::find_by_pid(tenant_pid)
        .filter(main::entities::tenants::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to find tenant" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Tenant not found" }),
        ))?;

    let mut update_model: main::entities::tenants::ActiveModel = tenant.to_owned().into();
    update_model.deleted_at = Set(Some(Utc::now().naive_utc()));
    update_model.updated_at = Set(Utc::now().naive_utc());
    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to soft delete tenant: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to soft delete tenant"
                }),
            )
        })?;

    let api = ApiClient::new();
    let endpoint = format!("tenants/soft-delete/{}", tenant.sso_tenant_id);

    let response: SuccessResponse = api
        .call(&endpoint, &None, None::<&()>, Method::DELETE)
        .await
        .map_err(|err| {
            log::error!("Failed to destroy tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to destroy tenant" }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": response.message }),
    ))
}

pub async fn restore_tenant(
    app_state: &AppState,
    tenant_pid: Uuid,
) -> Result<ApiResponse, ApiResponse> {
    let tenant = main::entities::tenants::Entity::find_by_pid(tenant_pid)
        .filter(main::entities::tenants::Column::DeletedAt.is_not_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to find tenant" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Tenant not found" }),
        ))?;

    let mut update_model: main::entities::tenants::ActiveModel = tenant.to_owned().into();
    update_model.deleted_at = Set(None);
    update_model.updated_at = Set(Utc::now().naive_utc());
    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to restore tenant: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to restore tenant"
                }),
            )
        })?;

    let api = ApiClient::new();
    let endpoint = format!("tenants/restore/{}", tenant.sso_tenant_id);

    let response: SuccessResponse = api
        .call(&endpoint, &None, None::<&()>, Method::POST)
        .await
        .map_err(|err| {
            log::error!("Failed to restore tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to restore tenant" }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": response.message }),
    ))
}

pub async fn permanently_delete_tenant(
    app_state: &AppState,
    tenant_pid: Uuid,
) -> Result<ApiResponse, ApiResponse> {
    let tenant = main::entities::tenants::Entity::find_by_pid(tenant_pid)
        .filter(main::entities::tenants::Column::DeletedAt.is_not_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to find tenant" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Tenant not found" }),
        ))?;

    let result = main::entities::tenants::Entity::delete_by_id(tenant.id)
        .exec(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to permanently delete tenant: {}", err);

            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to permanently delete tenant"
                }),
            )
        })?;

    if result.rows_affected == 0 {
        return Ok(ApiResponse::new(
            404,
            json!({ "message": "Tenant not found" }),
        ));
    }

    let api = ApiClient::new();
    let endpoint = format!("tenants/permanent/{}", tenant.sso_tenant_id);

    let response: SuccessResponse = api
        .call(&endpoint, &None, None::<&()>, Method::DELETE)
        .await
        .map_err(|err| {
            log::error!("Failed to permanently delete tenant: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to permanently delete tenant" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": response.message }),
    ))
}
