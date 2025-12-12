use std::collections::HashMap;

use crate::{
    AppState,
    db::main::{
        self,
        entities::sea_orm_active_enums::SubscriptionStatus,
        migrations::sea_orm::{
            ActiveModelTrait, ColumnTrait, Condition, EntityTrait, JoinType, PaginatorTrait,
            QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
        },
    },
    handlers::admin::subscriptions::SubscriptionDTO,
    utils::{
        api_response::ApiResponse, jwt::get_tenant_id, pagination::PaginationParams,
        permission::has_permission, validator_error::ValidationError,
    },
};
use actix_web::{HttpRequest, web};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let (tenant_id, _, _) = get_tenant_id(&req, &app_state).await?;

    let fetch_all = query.all.unwrap_or(false);

    let mut stmt = main::entities::subscriptions::Entity::find()
        .filter(main::entities::subscriptions::Column::TenantId.eq(tenant_id))
        .join(
            JoinType::InnerJoin,
            main::entities::subscriptions::Relation::SubscriptionPlans1.def(),
        );

    if !has_permission("view_archived_subscriptions", &req).await? {
        stmt = stmt.filter(main::entities::subscriptions::Column::DeletedAt.is_null());
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
            .column(main::entities::subscriptions::Column::PlanId)
            .column_as(
                main::entities::subscription_plans::Column::Name,
                "plan_name",
            )
            .column(main::entities::subscriptions::Column::Status)
            .into_model::<SubscriptionDTO>()
            .all(&app_state.main_db)
            .await
            .map_err(|err| {
                log::error!("Failed to fetch subscriptions: {}", err);
                ApiResponse::new(500, json!({ "message": "Failed to fetch subscriptions" }))
            })?
            .iter()
            .map(|sub| {
                json!({
                    "pid": sub.pid,
                    "tenant_id": sub.tenant_id,
                    "plan_id": sub.plan_id,
                    "plan_name": sub.plan_name,
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
        .column(main::entities::subscriptions::Column::PlanId)
        .column_as(
            main::entities::subscription_plans::Column::Name,
            "plan_name",
        )
        .column(main::entities::subscriptions::Column::Status)
        .column(main::entities::subscriptions::Column::CreatedAt)
        .column(main::entities::subscriptions::Column::UpdatedAt)
        .into_model::<SubscriptionDTO>()
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
        .map(|sub| {
            json!({
                "pid": sub.pid,
                "tenant_id": sub.tenant_id,
                "plan_id": sub.plan_id,
                "plan_name": sub.plan_name,
                "status": sub.status,
                "created_at": sub.created_at,
                "updated_at": sub.updated_at,
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
    let (tenant_id, _, _) = get_tenant_id(&req, &app_state).await?;
    let pid = path.into_inner();

    let mut stmt = main::entities::subscriptions::Entity::find_by_pid(pid)
        .filter(main::entities::subscriptions::Column::TenantId.eq(tenant_id));

    if !has_permission("view_archived_subscriptions", &req).await? {
        stmt = stmt.filter(main::entities::subscriptions::Column::DeletedAt.is_null());
    }

    let subscription = stmt
        .join(
            JoinType::InnerJoin,
            main::entities::subscriptions::Relation::SubscriptionPlans1.def(),
        )
        .select_only()
        .column(main::entities::subscriptions::Column::Pid)
        .column(main::entities::subscriptions::Column::TenantId)
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
        .into_model::<SubscriptionDTO>()
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

    Ok(ApiResponse::new(
        200,
        json!({
            "subscription": {
                "pid": subscription.pid,
                "tenant_id": subscription.tenant_id,
                "plan_id": subscription.plan_id,
                "plan_name": subscription.plan_name,
                "status": subscription.status,
                "current_period_start": subscription.current_period_start,
                "current_period_end": subscription.current_period_end,
                "cancel_at_period_end": subscription.cancel_at_period_end,
                "cancelled_at": subscription.cancelled_at,
                "cancellation_reason": subscription.cancellation_reason,
                "custom_price": subscription.custom_price,
                "trial_days": subscription.trial_days,
                "max_facilities": subscription.max_facilities,
                "max_users": subscription.max_users,
                "max_patients_per_month": subscription.max_patients_per_month,
                "storage_gb": subscription.storage_gb,
                "api_rate_limit_per_hour": subscription.api_rate_limit_per_hour,
                "created_at": subscription.created_at,
                "updated_at": subscription.updated_at,
            },
            "message": "Subscription fetched successfully",
        }),
    ))
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct CreateSubscriptionDTO {
    pub plan_id: i32,
}

impl CreateSubscriptionDTO {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.plan_id <= 0 {
            errors.insert(
                "plan_id".to_string(),
                "Plan ID must be greater than 0".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }
}

pub async fn trial(
    app_state: web::Data<AppState>,
    data: web::Json<CreateSubscriptionDTO>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let (tenant_id, _, _) = get_tenant_id(&req, &app_state).await?;

    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    let start = Utc::now().naive_utc();

    let end = start + Duration::weeks(1);

    main::entities::subscriptions::ActiveModel {
        tenant_id: Set(tenant_id),
        plan_id: Set(data.plan_id),
        status: Set(SubscriptionStatus::Trial),
        current_period_start: Set(Some(start)),
        current_period_end: Set(Some(end)),
        trial_days: Set(Some(7)),
        max_facilities: Set(Some(1)),
        max_patients_per_month: Set(Some(100)),
        storage_gb: Set(Some(5)),
        api_rate_limit_per_hour: Set(Some(1000)),
        ..Default::default()
    }
    .insert(&app_state.main_db)
    .await
    .map_err(|err| ApiResponse::new(500, json!({ "message": err.to_string() })))?;

    Ok(ApiResponse::new(
        201,
        json!({ "message": "Subscription created successfully" }),
    ))
}

pub async fn cancel(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let (tenant_id, _, _) = get_tenant_id(&req, &app_state).await?;
    let sub_id = path.into_inner();

    let subscription = main::entities::subscriptions::Entity::find_by_pid(sub_id)
        .filter(main::entities::subscriptions::Column::TenantId.eq(tenant_id))
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
    update_model.status = Set(SubscriptionStatus::Cancelled);
    update_model.updated_at = Set(Utc::now().naive_utc());
    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update subscription: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to update subscription" }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": "Subscription updated successfully" }),
    ))
}
