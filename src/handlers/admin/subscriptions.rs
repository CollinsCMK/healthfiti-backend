use std::collections::HashMap;

use actix_web::{HttpRequest, web};
use chrono::{Duration, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        entities::sea_orm_active_enums::SubscriptionStatus,
        migrations::sea_orm::{
            ActiveModelTrait, ColumnTrait, Condition, EntityTrait, FromQueryResult, PaginatorTrait,
            QueryFilter, QueryOrder, QuerySelect, Set,
        },
    },
    utils::{
        api_response::ApiResponse, app_state::AppState, pagination::PaginationParams,
        permission::has_permission, validator_error::ValidationError,
    },
};

#[derive(FromQueryResult, Debug, Clone)]
pub struct SubscriptionDTO {
    pub pid: String,
    pub tenant_id: String,
    pub plan_id: String,
    pub status: String,
    pub current_period_start: Option<NaiveDateTime>,
    pub current_period_end: Option<NaiveDateTime>,
    pub cancel_at_period_end: bool,
    pub cancelled_at: Option<NaiveDateTime>,
    pub cancellation_reason: Option<String>,
    pub custom_price: Option<Decimal>,
    pub trial_days: Option<i32>,
    pub max_facilities: Option<i32>,
    pub max_users: Option<i32>,
    pub max_patients_per_month: Option<i32>,
    pub storage_gb: Option<i32>,
    pub api_rate_limit_per_hour: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(FromQueryResult, Debug, Clone)]
pub struct TenantDTO {
    pub tenant_name: String,
}

#[derive(FromQueryResult, Debug, Clone)]
pub struct PlanDTO {
    pub plan_name: String,
}

pub async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let fetch_all = query.all.unwrap_or(false);

    let mut stmt = main::entities::subscriptions::Entity::find()
        .find_also_related(main::entities::subscription_plans::Entity)
        .find_also_related(main::entities::tenants::Entity);

    if !has_permission("view_archived_subscriptions", &req).await? {
        stmt = stmt.filter(main::entities::subscriptions::Column::DeletedAt.is_null());
    }

    if query.expiration.unwrap_or(false) {
        let now = Utc::now().naive_utc();
        let threshold = now + Duration::days(7);

        stmt = stmt
            .filter(main::entities::subscriptions::Column::CurrentPeriodEnd.lte(threshold))
            .filter(main::entities::subscriptions::Column::CurrentPeriodEnd.gte(now))
    }

    if let Some(term) = &query.search {
        use main::migrations::{Expr, extension::postgres::PgExpr};

        let like = format!("%{}%", term);
        stmt = stmt.filter(
            Condition::any()
                .add(Expr::col(main::entities::tenants::Column::Name).ilike(like.clone()))
                .add(
                    Expr::col(main::entities::subscription_plans::Column::Name).ilike(like.clone()),
                ),
        );
    }

    if fetch_all {
        let result = stmt
            .order_by_asc(main::entities::subscriptions::Column::CreatedAt)
            .select_only()
            .column(main::entities::subscriptions::Column::Pid)
            .column(main::entities::subscriptions::Column::TenantId)
            .column_as(main::entities::tenants::Column::Name, "tenant_name")
            .column(main::entities::subscriptions::Column::PlanId)
            .column_as(
                main::entities::subscription_plans::Column::Name,
                "plan_name",
            )
            .column(main::entities::subscriptions::Column::Status)
            .into_model::<SubscriptionDTO, PlanDTO, TenantDTO>()
            .all(&app_state.main_db)
            .await
            .map_err(|err| {
                log::error!("Failed to fetch subscriptions: {}", err);
                ApiResponse::new(500, json!({ "message": "Failed to fetch subscriptions" }))
            })?
            .iter()
            .map(|(sub, plan, tenant)| {
                json!({
                    "pid": sub.pid,
                    "tenant_id": sub.tenant_id,
                    "tenant_name": tenant.as_ref().map(|t| t.tenant_name.clone()),
                    "plan_id": sub.plan_id,
                    "plan_name": plan.as_ref().map(|p| p.plan_name.clone()),
                    "status": sub.status,
                })
            })
            .collect::<Vec<_>>();

        return Ok(ApiResponse::new(
            200,
            json!({
                "subscriptions": result,
                "message": "Subscriptions fetched successfully"
            }),
        ));
    }

    let page = query.page.unwrap_or(1).min(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let paginator = stmt
        .select_only()
        .column(main::entities::subscriptions::Column::Pid)
        .column(main::entities::subscriptions::Column::TenantId)
        .column_as(main::entities::tenants::Column::Name, "tenant_name")
        .column(main::entities::subscriptions::Column::PlanId)
        .column_as(
            main::entities::subscription_plans::Column::Name,
            "plan_name",
        )
        .column(main::entities::subscriptions::Column::Status)
        .column(main::entities::subscriptions::Column::CreatedAt)
        .column(main::entities::subscriptions::Column::UpdatedAt)
        .column(main::entities::subscriptions::Column::DeletedAt)
        .into_model::<SubscriptionDTO, PlanDTO, TenantDTO>()
        .paginate(&app_state.main_db, limit);

    let total_items = paginator
        .num_items()
        .await
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    let total_pages = (total_items as f64 / limit as f64).ceil() as u64;
    let has_prev = page > 1;
    let has_next = page < total_pages;

    let results = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .map_err(|err| {
            log::error!("Failed to fetch subscriptions: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to fetch subscriptions" }))
        })?
        .into_iter()
        .map(|(sub, plan, tenant)| {
            json!({
                "pid": sub.pid,
                "tenant_id": sub.tenant_id,
                "tenant_name": tenant.as_ref().map(|t| t.tenant_name.clone()),
                "plan_id": sub.plan_id,
                "plan_name": plan.as_ref().map(|p| p.plan_name.clone()),
                "status": sub.status,
                "created_at": sub.created_at,
                "updated_at": sub.updated_at,
                "deleted_at": sub.deleted_at,
            })
        })
        .collect::<Vec<_>>();

    Ok(ApiResponse::new(
        200,
        json!({
            "subscriptions": results,
            "page": page,
            "total_pages": total_pages,
            "total_items": total_items,
            "has_prev": has_prev,
            "has_next": has_next,
            "message": "Subscriptions fetched successfully",
        }),
    ))
}

pub async fn show(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let mut stmt = main::entities::subscriptions::Entity::find_by_pid(pid);

    if !has_permission("view_archived_subscriptions", &req).await? {
        stmt = stmt.filter(main::entities::subscriptions::Column::DeletedAt.is_null());
    }

    let subscription = stmt
        .find_also_related(main::entities::subscription_plans::Entity)
        .find_also_related(main::entities::tenants::Entity)
        .select_only()
        .column(main::entities::subscriptions::Column::Pid)
        .column(main::entities::subscriptions::Column::TenantId)
        .column(main::entities::tenants::Column::Name)
        .column(main::entities::subscriptions::Column::PlanId)
        .column(main::entities::subscription_plans::Column::Name)
        .column(main::entities::subscriptions::Column::Status)
        .column(main::entities::subscriptions::Column::CurrentPeriodStart)
        .column(main::entities::subscriptions::Column::CurrentPeriodEnd)
        .column(main::entities::subscriptions::Column::CancelAtPeriodEnd)
        .column(main::entities::subscriptions::Column::CancelledAt)
        .column(main::entities::subscriptions::Column::CancellationReason)
        .column(main::entities::subscriptions::Column::CustomPrice)
        .column(main::entities::subscriptions::Column::TrialDays)
        .column(main::entities::subscriptions::Column::MaxFacilities)
        .column(main::entities::subscriptions::Column::MaxUsers)
        .column(main::entities::subscriptions::Column::MaxPatientsPerMonth)
        .column(main::entities::subscriptions::Column::StorageGb)
        .column(main::entities::subscriptions::Column::ApiRateLimitPerHour)
        .column(main::entities::subscriptions::Column::CreatedAt)
        .column(main::entities::subscriptions::Column::UpdatedAt)
        .column(main::entities::subscriptions::Column::DeletedAt)
        .into_model::<SubscriptionDTO, PlanDTO, TenantDTO>()
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch subscription: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to fetch subscription" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Subscription not found" }),
        ))?;

    let (sub, plan, tenant) = subscription;

    Ok(ApiResponse::new(
        200,
        json!({
            "subscription": {
                "pid": sub.pid,
                "tenant_id": sub.tenant_id,
                "tenant_name": tenant.as_ref().map(|t| t.tenant_name.clone()),
                "plan_id": sub.plan_id,
                "plan_name": plan.as_ref().map(|p| p.plan_name.clone()),
                "status": sub.status,
                "current_period_start": sub.current_period_start,
                "current_period_end": sub.current_period_end,
                "cancel_at_period_end": sub.cancel_at_period_end,
                "cancelled_at": sub.cancelled_at,
                "cancellation_reason": sub.cancellation_reason,
                "custom_price": sub.custom_price,
                "trial_days": sub.trial_days,
                "max_facilities": sub.max_facilities,
                "max_users": sub.max_users,
                "max_patients_per_month": sub.max_patients_per_month,
                "storage_gb": sub.storage_gb,
                "api_rate_limit_per_hour": sub.api_rate_limit_per_hour,
                "created_at": sub.created_at,
                "updated_at": sub.updated_at,
                "deleted_at": sub.deleted_at,
            },
            "message": "Subscription fetched successfully",
        }),
    ))
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct SubscriptionStatusData {
    status: Option<String>,
}

impl SubscriptionStatusData {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if let Some(status) = &self.status {
            let status_lower = status.to_lowercase();
            if !["trial", "active", "past_due", "canceled", "expired"]
                .contains(&status_lower.as_str())
            {
                errors.insert(
                    "status".into(),
                    "Status must be one of: active, past_due, canceled, trial, expired.".into(),
                );
            }
        } else {
            errors.insert("status".into(), "Status is required.".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }

    pub fn get_status(&self) -> SubscriptionStatus {
        match self.status.as_ref().map(|s| s.to_lowercase()).as_deref() {
            Some("trial") => SubscriptionStatus::Trial,
            Some("active") => SubscriptionStatus::Active,
            Some("past_due") => SubscriptionStatus::PastDue,
            Some("cancelled") => SubscriptionStatus::Cancelled,
            Some("expired") => SubscriptionStatus::Expired,
            _ => SubscriptionStatus::Trial,
        }
    }
}

pub async fn set_active_status(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    data: web::Json<SubscriptionStatusData>,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let pid = path.into_inner();

    let subscription = main::entities::subscriptions::Entity::find_by_pid(pid)
        .filter(main::entities::subscriptions::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch subscription: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to fetch subscription" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Subscription not found" }),
        ))?;

    let mut update_model: main::entities::subscriptions::ActiveModel =
        subscription.to_owned().into();
    let mut changed = false;

    if subscription.status != data.get_status() {
        update_model.status = Set(data.get_status());
        changed = true;
    }

    if !changed {
        return Ok(ApiResponse::new(
            200,
            json!({ "message": "No updates were made because the data is unchanged." }),
        ));
    }

    update_model.updated_at = Set(Utc::now().naive_utc());
    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update subscription status: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to update subscription status" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": "Subscription status updated successfully" }),
    ))
}
