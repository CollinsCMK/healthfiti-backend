use std::collections::HashMap;

use actix_web::{HttpRequest, web};
use chrono::{NaiveDateTime, Utc};
use rust_decimal::{Decimal, dec};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        entities::sea_orm_active_enums::BillingCycle,
        migrations::sea_orm::{
            ActiveModelTrait, ColumnTrait, Condition, EntityTrait, FromQueryResult, PaginatorTrait,
            QueryFilter, QueryOrder, QuerySelect, Set,
        },
    },
    utils::{
        api_response::ApiResponse, app_state::AppState, pagination::PaginationParams,
        permission::has_permission, slug::slugify, validator_error::ValidationError,
    },
};

#[derive(FromQueryResult, Debug, Clone)]
pub struct SubscriptionPlansResultData {
    pub pid: Option<Uuid>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub price_weekly: Option<Decimal>,
    pub price_monthly: Option<Decimal>,
    pub price_quarterly: Option<Decimal>,
    pub price_yearly: Option<Decimal>,
    pub trial_days: Option<i32>,
    pub max_facilities: Option<i32>,
    pub max_users: Option<i32>,
    pub max_patients_per_month: Option<i32>,
    pub storage_gb: Option<i32>,
    pub api_rate_limit_per_hour: Option<i32>,
    pub billing_cycle: Option<String>,
    pub setup_fee: Option<Decimal>,
    pub is_public: Option<bool>,
    pub is_active: Option<bool>,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

pub async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let fetch_all = query.all.unwrap_or(false);

    let mut stmt = main::entities::subscription_plans::Entity::find();

    if !has_permission("view_archived_subscription_plans", &req).await? {
        stmt = stmt.filter(main::entities::subscription_plans::Column::DeletedAt.is_null());
    }

    if let Some(term) = &query.search {
        use main::migrations::{Expr, extension::postgres::PgExpr};

        let like = format!("%{}%", term);
        stmt = stmt.filter(
            Condition::any()
                .add(
                    Expr::col(main::entities::subscription_plans::Column::Name).ilike(like.clone()),
                )
                .add(
                    Expr::col(main::entities::subscription_plans::Column::Description)
                        .ilike(like.clone()),
                ),
        );
    }

    if fetch_all {
        let result = stmt
            .order_by_asc(main::entities::subscription_plans::Column::CreatedAt)
            .select_only()
            .column(main::entities::subscription_plans::Column::Pid)
            .column(main::entities::subscription_plans::Column::Name)
            .into_model::<SubscriptionPlansResultData>()
            .all(&app_state.main_db)
            .await
            .map_err(|err| {
                log::error!("Failed to fetch subscription plans: {}", err);
                ApiResponse::new(
                    500,
                    json!({ "message": "Failed to fetch subscription plans" }),
                )
            })?
            .iter()
            .map(|plan| {
                json!({
                    "pid": plan.pid,
                    "name": plan.name,
                })
            })
            .collect::<Vec<_>>();

        return Ok(ApiResponse::new(
            200,
            json!({
                "subscription_plans": result,
                "message": "Subscription plans fetched successfully"
            }),
        ));
    }

    let page = query.page.unwrap_or(1).min(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let paginator = stmt
        .select_only()
        .column(main::entities::subscription_plans::Column::Pid)
        .column(main::entities::subscription_plans::Column::Name)
        .column(main::entities::subscription_plans::Column::Slug)
        .column(main::entities::subscription_plans::Column::IsActive)
        .column(main::entities::subscription_plans::Column::CreatedAt)
        .column(main::entities::subscription_plans::Column::UpdatedAt)
        .column(main::entities::subscription_plans::Column::DeletedAt)
        .into_model::<SubscriptionPlansResultData>()
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
        .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?
        .into_iter()
        .map(|plan| {
            json!({
                "pid": plan.pid,
                "name": plan.name,
                "slug": plan.slug,
                "is_active": plan.is_active,
                "created_at": plan.created_at,
                "updated_at": plan.updated_at,
                "deleted_at": plan.deleted_at,
            })
        })
        .collect::<Vec<_>>();

    Ok(ApiResponse::new(
        200,
        json!({
            "subscription_plans": results,
            "page": page,
            "total_pages": total_pages,
            "total_items": total_items,
            "has_prev": has_prev,
            "has_next": has_next,
            "message": "Subscription plans fetched successfully",
        }),
    ))
}

pub async fn show(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let mut stmt = main::entities::subscription_plans::Entity::find_by_pid(pid);

    if !has_permission("view_archived_subscription_plans", &req).await? {
        stmt = stmt.filter(main::entities::subscription_plans::Column::DeletedAt.is_null());
    }

    let subscription_plan = stmt
        .select_only()
        .column(main::entities::subscription_plans::Column::Pid)
        .column(main::entities::subscription_plans::Column::Name)
        .column(main::entities::subscription_plans::Column::Slug)
        .column(main::entities::subscription_plans::Column::Description)
        .column(main::entities::subscription_plans::Column::PriceWeekly)
        .column(main::entities::subscription_plans::Column::PriceMonthly)
        .column(main::entities::subscription_plans::Column::PriceQuarterly)
        .column(main::entities::subscription_plans::Column::PriceYearly)
        .column(main::entities::subscription_plans::Column::TrialDays)
        .column(main::entities::subscription_plans::Column::MaxFacilities)
        .column(main::entities::subscription_plans::Column::MaxUsers)
        .column(main::entities::subscription_plans::Column::MaxPatientsPerMonth)
        .column(main::entities::subscription_plans::Column::StorageGb)
        .column(main::entities::subscription_plans::Column::ApiRateLimitPerHour)
        .column(main::entities::subscription_plans::Column::BillingCycle)
        .column(main::entities::subscription_plans::Column::SetupFee)
        .column(main::entities::subscription_plans::Column::IsPublic)
        .column(main::entities::subscription_plans::Column::IsActive)
        .column(main::entities::subscription_plans::Column::CreatedAt)
        .column(main::entities::subscription_plans::Column::UpdatedAt)
        .column(main::entities::subscription_plans::Column::DeletedAt)
        .into_model::<SubscriptionPlansResultData>()
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch subscription plan: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to fetch subscription plan" }),
            )
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Subscription plan not found" }),
        ))?;

    Ok(ApiResponse::new(
        200,
        json!({
            "subscription_plan": {
                "pid": subscription_plan.pid,
                "name": subscription_plan.name,
                "slug": subscription_plan.slug,
                "description": subscription_plan.description,
                "price_weekly": subscription_plan.price_weekly,
                "price_monthly": subscription_plan.price_monthly,
                "price_quarterly": subscription_plan.price_quarterly,
                "price_yearly": subscription_plan.price_yearly,
                "trial_days": subscription_plan.trial_days,
                "max_facilities": subscription_plan.max_facilities,
                "max_users": subscription_plan.max_users,
                "max_patients_per_month": subscription_plan.max_patients_per_month,
                "storage_gb": subscription_plan.storage_gb,
                "api_rate_limit_per_hour": subscription_plan.api_rate_limit_per_hour,
                "billing_cycle": subscription_plan.billing_cycle,
                "setup_fee": subscription_plan.setup_fee,
                "is_public": subscription_plan.is_public,
                "is_active": subscription_plan.is_active,
                "created_at": subscription_plan.created_at,
                "updated_at": subscription_plan.updated_at,
                "deleted_at": subscription_plan.deleted_at,
            },
            "message": "Subscription plan fetched successfully",
        }),
    ))
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct CreateSubscriptionPlan {
    pub name: String,
    pub description: Option<String>,
    pub price_weekly: Option<Decimal>,
    pub price_monthly: Option<Decimal>,
    pub price_quarterly: Option<Decimal>,
    pub price_yearly: Option<Decimal>,
    pub trial_days: i32,
    pub max_facilities: Option<i32>,
    pub max_users: Option<i32>,
    pub max_patients_per_month: Option<i32>,
    pub storage_gb: Option<i32>,
    pub api_rate_limit_per_hour: Option<i32>,
    pub billing_cycle: Option<String>,
    pub setup_fee: Decimal,
    pub is_public: bool,
    pub is_active: bool,
}

impl CreateSubscriptionPlan {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.name.is_empty() {
            errors.insert("name".to_string(), "Name is required".to_string());
        }

        if self.trial_days < 0 {
            errors.insert(
                "trial_days".to_string(),
                "Trial days must be greater than or equal to 0".to_string(),
            );
        }

        if let Some(billing_cycle) = &self.billing_cycle {
            let billing_cycle_lower = billing_cycle.to_lowercase();
            if !["weekly", "monthly", "quarterly", "yearly", "custom"]
                .contains(&billing_cycle_lower.as_str())
            {
                errors.insert(
                    "billing_cycle".into(),
                    "Billing cycle must be one of: weekly, monthly, quarterly, yearly, custom."
                        .into(),
                );
            }
        } else {
            errors.insert("billing_cycle".into(), "Billing cycle is required.".into());
        }

        if self.setup_fee < dec!(0) {
            errors.insert(
                "setup_fee".to_string(),
                "Setup fee must be greater than or equal to 0".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }

    pub fn get_billing_cycle(&self) -> BillingCycle {
        match self
            .billing_cycle
            .as_ref()
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("weekly") => BillingCycle::Weekly,
            Some("monthly") => BillingCycle::Monthly,
            Some("quarterly") => BillingCycle::Quarterly,
            Some("yearly") => BillingCycle::Yearly,
            Some("custom") => BillingCycle::Custom,
            _ => BillingCycle::Monthly,
        }
    }
}

pub async fn create(
    app_state: web::Data<AppState>,
    data: web::Json<CreateSubscriptionPlan>,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let slug = slugify(&data.name);

    let _ = main::entities::subscription_plans::ActiveModel {
        name: Set(data.name.clone()),
        slug: Set(slug),
        description: Set(data.description.clone()),
        price_weekly: Set(data.price_weekly),
        price_monthly: Set(data.price_monthly),
        price_quarterly: Set(data.price_quarterly),
        price_yearly: Set(data.price_yearly),
        trial_days: Set(data.trial_days),
        max_facilities: Set(data.max_facilities),
        max_users: Set(data.max_users),
        max_patients_per_month: Set(data.max_patients_per_month),
        storage_gb: Set(data.storage_gb),
        api_rate_limit_per_hour: Set(data.api_rate_limit_per_hour),
        billing_cycle: Set(data.get_billing_cycle()),
        setup_fee: Set(data.setup_fee),
        is_public: Set(data.is_public),
        is_active: Set(data.is_active),
        ..Default::default()
    }
    .insert(&app_state.main_db)
    .await
    .map_err(|err| {
        log::error!("Failed to create subscription plan: {}", err);
        ApiResponse::new(
            500,
            json!({ "message": "Failed to create subscription plan" }),
        )
    })?;

    Ok(ApiResponse::new(
        201,
        json!({
            "message": "Subscription plan created successfully",
        }),
    ))
}

pub async fn edit(
    app_state: web::Data<AppState>,
    data: web::Json<CreateSubscriptionPlan>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(500, json!(err)));
    }

    let pid = path.into_inner();

    let subscription_plan = main::entities::subscription_plans::Entity::find_by_pid(pid)
        .filter(main::entities::subscription_plans::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find subscription plan: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to find subscription plan" }),
            )
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Subscription plan not found" }),
        ))?;

    let mut update_model: main::entities::subscription_plans::ActiveModel =
        subscription_plan.to_owned().into();
    let mut changed = false;

    if data.name != subscription_plan.name {
        update_model.name = Set(data.name.clone());
        changed = true;
    }

    if data.description != subscription_plan.description {
        update_model.description = Set(data.description.clone());
        changed = true;
    }

    if data.price_weekly != subscription_plan.price_weekly {
        update_model.price_weekly = Set(data.price_weekly);
        changed = true;
    }

    if data.price_monthly != subscription_plan.price_monthly {
        update_model.price_monthly = Set(data.price_monthly);
        changed = true;
    }

    if data.price_quarterly != subscription_plan.price_quarterly {
        update_model.price_quarterly = Set(data.price_quarterly);
        changed = true;
    }

    if data.price_yearly != subscription_plan.price_yearly {
        update_model.price_yearly = Set(data.price_yearly);
        changed = true;
    }

    if data.trial_days != subscription_plan.trial_days {
        update_model.trial_days = Set(data.trial_days);
        changed = true;
    }

    if data.max_facilities != subscription_plan.max_facilities {
        update_model.max_facilities = Set(data.max_facilities);
        changed = true;
    }

    if data.max_users != subscription_plan.max_users {
        update_model.max_users = Set(data.max_users);
        changed = true;
    }

    if data.max_patients_per_month != subscription_plan.max_patients_per_month {
        update_model.max_patients_per_month = Set(data.max_patients_per_month);
        changed = true;
    }

    if data.storage_gb != subscription_plan.storage_gb {
        update_model.storage_gb = Set(data.storage_gb);
        changed = true;
    }

    if data.api_rate_limit_per_hour != subscription_plan.api_rate_limit_per_hour {
        update_model.api_rate_limit_per_hour = Set(data.api_rate_limit_per_hour);
        changed = true;
    }

    if data.get_billing_cycle() != subscription_plan.billing_cycle {
        update_model.billing_cycle = Set(data.get_billing_cycle());
        changed = true;
    }

    if data.setup_fee != subscription_plan.setup_fee {
        update_model.setup_fee = Set(data.setup_fee);
        changed = true;
    }

    if data.is_public != subscription_plan.is_public {
        update_model.is_public = Set(data.is_public);
        changed = true;
    }

    if data.is_active != subscription_plan.is_active {
        update_model.is_active = Set(data.is_active);
        changed = true;
    }

    if !changed {
        return Ok(ApiResponse::new(
            200,
            json!({
                "message": "No updates were made because the data is unchanged.",
            }),
        ));
    }

    update_model.updated_at = Set(Utc::now().naive_utc());
    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update subscription plan: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to update subscription plan" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": "Subscription plan updated successfully",
        }),
    ))
}

pub async fn set_active_status(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let subscription_plan = main::entities::subscription_plans::Entity::find_by_pid(pid)
        .filter(main::entities::subscription_plans::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find subscription plan: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to find subscription plan" }),
            )
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Subscription plan not found" }),
        ))?;

    let mut update_model: main::entities::subscription_plans::ActiveModel =
        subscription_plan.to_owned().into();
    let is_active = subscription_plan.is_active;
    let new_status = !is_active;

    update_model.is_active = Set(new_status);
    update_model.updated_at = Set(Utc::now().naive_utc());
    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update subscription plan: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to update subscription plan" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": format!("Subscription plan {} successfully", if new_status { "activated" } else { "deactivated" }),
        }),
    ))
}

pub async fn destroy(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let subscription_plan = main::entities::subscription_plans::Entity::find_by_pid(pid)
        .filter(main::entities::subscription_plans::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find subscription plan: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to find subscription plan" }),
            )
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Subscription plan not found" }),
        ))?;

    let mut update_model: main::entities::subscription_plans::ActiveModel =
        subscription_plan.to_owned().into();
    update_model.deleted_at = Set(Some(Utc::now().naive_utc()));
    update_model.updated_at = Set(Utc::now().naive_utc());
    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update subscription plan: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to update subscription plan" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": "Subscription plan deleted successfully",
        }),
    ))
}

pub async fn restore(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let subscription_plan = main::entities::subscription_plans::Entity::find_by_pid(pid)
        .filter(main::entities::subscription_plans::Column::DeletedAt.is_not_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find subscription plan: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to find subscription plan" }),
            )
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Subscription plan not found" }),
        ))?;

    let mut update_model: main::entities::subscription_plans::ActiveModel =
        subscription_plan.to_owned().into();
    update_model.deleted_at = Set(None);
    update_model.updated_at = Set(Utc::now().naive_utc());
    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update subscription plan: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to update subscription plan" }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": "Subscription plan restored successfully",
        }),
    ))
}

pub async fn delete_permanently(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let subscription_plan = main::entities::subscription_plans::Entity::find_by_pid(pid)
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find subscription plan: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to find subscription plan" }),
            )
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Subscription plan not found" }),
        ))?;

    let result = main::entities::subscription_plans::Entity::delete_by_id(subscription_plan.id)
        .exec(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to delete subscription plan: {}", err);
            ApiResponse::new(
                500,
                json!({ "message": "Failed to delete subscription plan" }),
            )
        })?;

    if result.rows_affected == 0 {
        return Err(ApiResponse::new(
            404,
            json!({ "message": "Subscription plan not found" }),
        ));
    }

    Ok(ApiResponse::new(
        200,
        json!({
            "message": "Subscription plan permanently deleted successfully",
        }),
    ))
}
