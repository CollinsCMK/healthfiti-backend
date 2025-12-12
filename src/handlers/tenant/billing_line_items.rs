use actix_web::{HttpRequest, web};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::main::{
        self,
        migrations::sea_orm::{
            ColumnTrait, Condition, EntityTrait, PaginatorTrait,
            QueryFilter, QuerySelect,
        },
    }, handlers::admin::billing_line_items::BillingLineItemData, utils::{
        api_response::ApiResponse, app_state::AppState, jwt::get_tenant_id, pagination::PaginationParams, permission::has_permission
    }
};

pub async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let (tenant_id, _, _) = get_tenant_id(&req, &app_state).await?;

    let mut stmt = main::entities::billing_line_items::Entity::find()
        .filter(main::entities::billing_line_items::Column::TenantId.eq(tenant_id));

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

pub async fn show(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let (tenant_id, _, _) = get_tenant_id(&req, &app_state).await?;

    let pid = path.into_inner();

    let mut stmt = main::entities::billing_line_items::Entity::find_by_pid(pid)
        .filter(main::entities::billing_line_items::Column::TenantId.eq(tenant_id));

    if !has_permission("view_archived_billing_line_items", &req).await? {
        stmt = stmt.filter(main::entities::billing_line_items::Column::DeletedAt.is_null());
    }

    let result = stmt
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
        .column(main::entities::billing_line_items::Column::CreatedAt)
        .column(main::entities::billing_line_items::Column::UpdatedAt)
        .into_model::<BillingLineItemData>()
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
                "pid": result.pid,
                "billing_period_start": result.billing_period_start,
                "billing_period_end": result.billing_period_end,
                "description": result.description,
                "quantity": result.quantity,
                "unit_price": result.unit_price,
                "total_amount": result.total_amount,
                "item_type": result.item_type,
                "metadata": result.metadata,
                "created_at": result.created_at,
                "updated_at": result.updated_at,
            },
            "message": "Billing line item fetched successfully",
        }),
    ))
}
