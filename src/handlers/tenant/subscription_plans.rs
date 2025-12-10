use actix_web::{HttpRequest, web};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{
            ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
            QuerySelect,
        },
    },
    handlers::admin::subscription_plans::SubscriptionPlansResultData,
    utils::{
        api_response::ApiResponse, app_state::AppState, pagination::PaginationParams,
        permission::has_permission,
    },
};

pub async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let fetch_all = query.all.unwrap_or(false);

    let mut stmt = main::entities::subscription_plans::Entity::find()
        .filter(main::entities::subscription_plans::Column::IsActive.eq(true));

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
        .column(main::entities::subscription_plans::Column::CreatedAt)
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
                "created_at": plan.created_at,
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

    let mut stmt = main::entities::subscription_plans::Entity::find_by_pid(pid)
        .filter(main::entities::subscription_plans::Column::IsActive.eq(true));

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
        .column(main::entities::subscription_plans::Column::CreatedAt)
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
                "created_at": subscription_plan.created_at,
            },
            "message": "Subscription plan fetched successfully",
        }),
    ))
}
