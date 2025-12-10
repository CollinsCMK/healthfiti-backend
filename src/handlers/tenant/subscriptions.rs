use std::collections::HashMap;

use crate::{
    AppState,
    db::main::{
        self,
        entities::sea_orm_active_enums::{BillingCycle, SubscriptionStatus},
        migrations::sea_orm::{
            ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter,
            QueryOrder, QuerySelect, Set,
        },
    },
    handlers::admin::subscriptions::{PlanDTO, SubscriptionDTO},
    utils::{
        api_response::ApiResponse, jwt::get_tenant_id, pagination::PaginationParams,
        permission::has_permission, validator_error::ValidationError,
    },
};
use actix_web::{HttpRequest, web};
use chrono::{Duration, NaiveDateTime, Utc};
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
        .find_also_related(main::entities::subscription_plans::Entity);

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
            .into_model::<SubscriptionDTO, PlanDTO>()
            .all(&app_state.main_db)
            .await
            .map_err(|err| {
                log::error!("Failed to fetch subscriptions: {}", err);
                ApiResponse::new(500, json!({ "message": "Failed to fetch subscriptions" }))
            })?
            .iter()
            .map(|(sub, plan)| {
                json!({
                    "pid": sub.pid,
                    "tenant_id": sub.tenant_id,
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
        .column(main::entities::subscriptions::Column::PlanId)
        .column_as(
            main::entities::subscription_plans::Column::Name,
            "plan_name",
        )
        .column(main::entities::subscriptions::Column::Status)
        .column(main::entities::subscriptions::Column::CreatedAt)
        .column(main::entities::subscriptions::Column::UpdatedAt)
        .into_model::<SubscriptionDTO, PlanDTO>()
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
        .map(|(sub, plan)| {
            json!({
                "pid": sub.pid,
                "tenant_id": sub.tenant_id,
                "plan_id": sub.plan_id,
                "plan_name": plan.as_ref().map(|p| p.plan_name.clone()),
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

fn compute_period_end(
    start: NaiveDateTime,
    cycle: &BillingCycle,
    custom_days: Option<i32>,
) -> NaiveDateTime {
    match cycle {
        BillingCycle::Weekly => start + Duration::weeks(1),
        BillingCycle::Monthly => start + Duration::days(30),
        BillingCycle::Quarterly => start + Duration::days(90),
        BillingCycle::Yearly => start + Duration::days(365),
        BillingCycle::Custom => start + Duration::days(custom_days.unwrap_or(0) as i64),
    }
}

pub async fn trial(
    app_state: web::Data<AppState>,
    data: web::Json<CreateSubscriptionDTO>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let (tenant_id, _, _) = get_tenant_id(&req, &app_state).await?;

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

pub async fn create(
    app_state: web::Data<AppState>,
    data: web::Json<CreateSubscriptionDTO>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let (tenant_id, _, _) = get_tenant_id(&req, &app_state).await?;

    let plan = main::entities::subscription_plans::Entity::find_by_id(data.plan_id)
        .filter(main::entities::subscription_plans::Column::DeletedAt.is_null())
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

    let start = Utc::now().naive_utc();
    
    let end = compute_period_end(
        start,
        &plan.billing_cycle,
        None,
    );

    let existing_sub = main::entities::subscriptions::Entity::find()
        .filter(main::entities::subscriptions::Column::TenantId.eq(tenant_id))
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch subscription: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    if let Some(existing) = existing_sub.clone() {
        if existing.status == SubscriptionStatus::Active || existing.status == SubscriptionStatus::Trial {
            return Err(ApiResponse::new(
                409,
                json!({ "message": "Tenant already has a subscription" }),
            ));
        }
    }

    let give_trial = existing_sub.is_none() && plan.trial_days > 0;

    let status = if give_trial {
        SubscriptionStatus::Trial
    } else {
        SubscriptionStatus::Active
    };

    main::entities::subscriptions::ActiveModel {
        tenant_id: Set(tenant_id),
        plan_id: Set(plan.id),
        status: Set(status),
        current_period_start: Set(Some(start)),
        current_period_end: Set(Some(end)),
        trial_days: Set(Some(plan.trial_days)),
        max_facilities: Set(plan.max_facilities),
        max_patients_per_month: Set(plan.max_patients_per_month),
        storage_gb: Set(plan.storage_gb),
        api_rate_limit_per_hour: Set(plan.api_rate_limit_per_hour),
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

pub fn compute_period_duration(
    cycle: &BillingCycle,
    custom_days: Option<i32>,
) -> Duration {
    match cycle {
        BillingCycle::Weekly => Duration::days(7),
        BillingCycle::Monthly => Duration::days(30),
        BillingCycle::Quarterly => Duration::days(90),
        BillingCycle::Yearly => Duration::days(365),
        BillingCycle::Custom => Duration::days(custom_days.unwrap_or(0) as i64),
    }
}

pub async fn update(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    data: web::Json<CreateSubscriptionDTO>,
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
        .ok_or(ApiResponse::new(404, json!({ "message": "Subscription not found" })))?;
    
    let plan = main::entities::subscription_plans::Entity::find_by_id(data.plan_id)
        .filter(main::entities::subscription_plans::Column::DeletedAt.is_null())
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

    let start = Utc::now().naive_utc();
    
    let current_end = subscription.current_period_end.unwrap_or(start);
    let new_end = if current_end > start {
        current_end + compute_period_duration(&plan.billing_cycle, None)
    } else {
        start + compute_period_duration(&plan.billing_cycle, None)
    };

    let mut update_model: main::entities::subscriptions::ActiveModel = subscription.to_owned().into();
    update_model.plan_id = Set(plan.id);
    update_model.status = Set(SubscriptionStatus::Active);
    update_model.current_period_start = Set(Some(start));
    update_model.current_period_end = Set(Some(new_end));
    update_model.trial_days = Set(Some(plan.trial_days));
    update_model.max_facilities = Set(plan.max_facilities);
    update_model.max_patients_per_month = Set(plan.max_patients_per_month);
    update_model.storage_gb = Set(plan.storage_gb);
    update_model.api_rate_limit_per_hour = Set(plan.api_rate_limit_per_hour);

    update_model.update(&app_state.main_db).await.map_err(|err| {
        log::error!("Failed to update subscription: {}", err);
        ApiResponse::new(500, json!({ "message": "Failed to update subscription" }))
    })?;

    

    Ok(ApiResponse::new(
        200,
        json!({ "message": "Subscription updated successfully" }),
    ))
}
