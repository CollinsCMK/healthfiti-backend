use actix_web::{HttpRequest, web};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{
            ActiveModelTrait, ColumnTrait, Condition, EntityTrait, FromQueryResult, PaginatorTrait,
            QueryFilter, QuerySelect, Set,
        },
    },
    utils::{
        api_response::ApiResponse, app_state::AppState, pagination::PaginationParams,
        permission::has_permission
    },
};

#[derive(FromQueryResult, Debug, Clone)]
pub struct BillingLineItemData {
    pub pid: Uuid,
    pub billing_period_start: Option<NaiveDate>,
    pub billing_period_end: Option<NaiveDate>,
    pub description: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub total_amount: f64,
    pub item_type: String,
    pub metadata: Option<Value>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub deleted_at: Option<NaiveDateTime>,
}

pub async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let mut stmt = main::entities::billing_line_items::Entity::find();

    if !has_permission("view_archived_billing_line_items", &req).await? {
        stmt = stmt.filter(main::entities::billing_line_items::Column::DeletedAt.is_null());
    }

    if let Some(term) = &query.search {
        use main::migrations::{Expr, extension::postgres::PgExpr};

        let like = format!("%{}%", term);
        stmt = stmt.filter(
            Condition::any()
                .add(
                    Expr::col(main::entities::billing_line_items::Column::Description)
                        .ilike(like.clone()),
                )
                .add(
                    Expr::col(main::entities::billing_line_items::Column::ItemType)
                        .ilike(like.clone()),
                ),
        );
    }

    let page = query.page.unwrap_or(1).min(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let paginator = stmt
        .select_only()
        .column(main::entities::billing_line_items::Column::Pid)
        .column(main::entities::billing_line_items::Column::Description)
        .column(main::entities::billing_line_items::Column::ItemType)
        .column(main::entities::billing_line_items::Column::CreatedAt)
        .column(main::entities::billing_line_items::Column::UpdatedAt)
        .column(main::entities::billing_line_items::Column::DeletedAt)
        .into_model::<BillingLineItemData>()
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
        .map(|item| {
            json!({
                "pid": item.pid,
                "description": item.description,
                "item_type": item.item_type,
                "created_at": item.created_at,
                "updated_at": item.updated_at,
                "deleted_at": item.deleted_at,
            })
        })
        .collect::<Vec<_>>();

    Ok(ApiResponse::new(
        200,
        json!({
            "billing_line_items": results,
            "page": page,
            "total_pages": total_pages,
            "total_items": total_items,
            "has_prev": has_prev,
            "has_next": has_next,
            "message": "Billing line items fetched successfully",
        }),
    ))
}

#[derive(FromQueryResult, Debug, Clone)]
pub struct TenantBillingLineItemData {
    tenant_pid: Option<Uuid>,
    name: Option<String>,
}

#[derive(FromQueryResult, Debug, Clone)]
pub struct SubscriptionBillingLineItemData {
    subscription_pid: Option<Uuid>,
    status: Option<String>,
}

pub async fn show(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let mut stmt = main::entities::billing_line_items::Entity::find_by_pid(pid)
        .find_also_related(main::entities::tenants::Entity)
        .find_also_related(main::entities::subscriptions::Entity);

    if !has_permission("view_archived_billing_line_items", &req).await? {
        stmt = stmt.filter(main::entities::billing_line_items::Column::DeletedAt.is_null());
    }

    let (item, tenant, subscription) = stmt
        .select_only()
        .column(main::entities::billing_line_items::Column::Pid)
        .column(main::entities::billing_line_items::Column::BillingPeriodStart)
        .column(main::entities::billing_line_items::Column::BillingPeriodEnd)
        .column(main::entities::billing_line_items::Column::Description)
        .column(main::entities::billing_line_items::Column::Quantity)
        .column(main::entities::billing_line_items::Column::UnitPrice)
        .column(main::entities::billing_line_items::Column::TotalAmount)
        .column(main::entities::billing_line_items::Column::ItemType)
        .column(main::entities::billing_line_items::Column::Metadata)
        .column_as(main::entities::tenants::Column::Pid, "tenant_pid")
        .column(main::entities::tenants::Column::Name)
        .column_as(main::entities::subscriptions::Column::Pid, "subscription_pid")
        .column(main::entities::subscriptions::Column::Status)
        .column(main::entities::billing_line_items::Column::CreatedAt)
        .column(main::entities::billing_line_items::Column::UpdatedAt)
        .column(main::entities::billing_line_items::Column::DeletedAt)
        .into_model::<BillingLineItemData, TenantBillingLineItemData, SubscriptionBillingLineItemData>()
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch billing line item: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?
        .ok_or_else(|| ApiResponse::new(404, json!({ "message": "Billing line item not found" })))?;

    Ok(ApiResponse::new(
        200,
        json!({
            "billing_line_item": {
                "pid": item.pid,
                "billing_period_start": item.billing_period_start,
                "billing_period_end": item.billing_period_end,
                "description": item.description,
                "quantity": item.quantity,
                "unit_price": item.unit_price,
                "total_amount": item.total_amount,
                "item_type": item.item_type,
                "metadata": item.metadata,
                "tenant_pid": tenant.as_ref().map(|t| t.tenant_pid),
                "tenant_name": tenant.as_ref().map(|t| t.name.clone()),
                "subscription_pid": subscription.as_ref().map(|s| s.subscription_pid),
                "subscription_status": subscription.as_ref().map(|s| s.status.clone()),
                "created_at": item.created_at,
                "updated_at": item.updated_at,
                "deleted_at": item.deleted_at,
            },
            "message": "Billing line item fetched successfully",
        }),
    ))
}

pub async fn destroy(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let billing_item = main::entities::billing_line_items::Entity::find_by_pid(pid)
        .filter(main::entities::billing_line_items::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch billing line item: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?
        .ok_or_else(|| ApiResponse::new(404, json!({ "message": "Billing line item not found" })))?;

    let mut active_model: main::entities::billing_line_items::ActiveModel = billing_item.to_owned().into();
    active_model.deleted_at = Set(Some(Utc::now().naive_utc()));
    active_model.updated_at = Set(Utc::now().naive_utc());
    active_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to destroy billing line item: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": "Billing line item destroyed successfully" }),
    ))
}

pub async fn restore(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let billing_item = main::entities::billing_line_items::Entity::find_by_pid(pid)
        .filter(main::entities::billing_line_items::Column::DeletedAt.is_not_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch billing line item: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?
        .ok_or_else(|| ApiResponse::new(404, json!({ "message": "Billing line item not found" })))?;

    let mut active_model: main::entities::billing_line_items::ActiveModel = billing_item.to_owned().into();
    active_model.deleted_at = Set(None);
    active_model.updated_at = Set(Utc::now().naive_utc());
    active_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to restore billing line item: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": "Billing line item restored successfully" }),
    ))
}

pub async fn delete_permanently(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let billing_item = main::entities::billing_line_items::Entity::find_by_pid(pid)
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch billing line item: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?
        .ok_or_else(|| ApiResponse::new(404, json!({ "message": "Billing line item not found" })))?;

    let result = main::entities::billing_line_items::Entity::delete_by_id(billing_item.id)
        .exec(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to delete permanently billing line item: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    if result.rows_affected == 0 {
        return Err(ApiResponse::new(
            404,
            json!({ "message": "Billing line item not found" }),
        ));
    }

    Ok(ApiResponse::new(
        200,
        json!({ "message": "Billing line item deleted permanently successfully" }),
    ))
}
