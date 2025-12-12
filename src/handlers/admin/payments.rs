use std::collections::HashMap;

use crate::{
    db::main::{
        self,
        entities::sea_orm_active_enums::{PaymentStatus, SubscriptionStatus},
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
use actix_web::{HttpRequest, web};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(FromQueryResult, Debug, Clone, Serialize)]
pub struct PaymentTransactionsResultData {
    pub pid: Uuid,
    pub tenant_id: i32,
    pub subscription_id: Option<i32>,
    pub amount: Decimal,
    pub currency: String,
    pub status: String,
    pub payment_method: String,
    pub invoice_url: Option<String>,
    pub description: Option<String>,
    pub failure_reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(FromQueryResult, Debug, Clone, Serialize)]
pub struct PaymentTenantResultData {
    pub tenant_pid: Uuid,
    pub tenant_name: String,
}

#[derive(FromQueryResult, Debug, Clone, Serialize)]
pub struct PaymentSubscriptionResultData {
    pub pid: Uuid,
    pub status: SubscriptionStatus,
}

pub async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let mut stmt = main::entities::payment_transactions::Entity::find();

    if !has_permission("view_archived_payment_transactions", &req).await? {
        stmt = stmt.filter(main::entities::payment_transactions::Column::DeletedAt.is_null());
    }

    if let Some(term) = &query.search {
        use main::migrations::{Expr, extension::postgres::PgExpr};

        let like = format!("%{}%", term);
        stmt = stmt.filter(
            Condition::any()
                .add(
                    Expr::col(main::entities::payment_transactions::Column::PaymentMethod)
                        .ilike(like.clone()),
                )
                .add(
                    Expr::col(main::entities::payment_transactions::Column::Description)
                        .ilike(like.clone()),
                ),
        );
    }

    let page = query.page.unwrap_or(1).min(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let paginator = stmt
        .select_only()
        .column(main::entities::payment_transactions::Column::Pid)
        .column(main::entities::payment_transactions::Column::Status)
        .column(main::entities::payment_transactions::Column::Metadata)
        .column(main::entities::payment_transactions::Column::CreatedAt)
        .column(main::entities::payment_transactions::Column::UpdatedAt)
        .column(main::entities::payment_transactions::Column::DeletedAt)
        .order_by_asc(main::entities::payment_transactions::Column::CreatedAt)
        .into_model::<PaymentTransactionsResultData>()
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
        .map(|p| {
            json!({
                "pid": p.pid,
                "status": p.status,
                "created_at": p.created_at,
                "updated_at": p.updated_at,
                "deleted_at": p.deleted_at,
            })
        })
        .collect::<Vec<_>>();

    Ok(ApiResponse::new(
        200,
        json!({
            "payment_transactions": results,
            "page": page,
            "total_pages": total_pages,
            "total_items": total_items,
            "has_prev": has_prev,
            "has_next": has_next,
            "message": "Payment transactions fetched successfully"
        }),
    ))
}

pub async fn show(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let mut stmt = main::entities::payment_transactions::Entity::find_by_pid(pid);

    if !has_permission("view_archived_payment_transactions", &req).await? {
        stmt = stmt.filter(main::entities::payment_transactions::Column::DeletedAt.is_null());
    }

    let (payment_transaction, tenant) = stmt
        .find_also_related(main::entities::tenants::Entity)
        .filter(main::entities::payment_transactions::Column::DeletedAt.is_null())
        .select_only()
        .column(main::entities::payment_transactions::Column::Pid)
        .column(main::entities::payment_transactions::Column::TenantId)
        .column_as(main::entities::tenants::Column::Pid, "tenant_pid")
        .column_as(main::entities::tenants::Column::Name, "tenant_name")
        .column(main::entities::payment_transactions::Column::SubscriptionId)
        .column(main::entities::payment_transactions::Column::Amount)
        .column(main::entities::payment_transactions::Column::Currency)
        .column(main::entities::payment_transactions::Column::Status)
        .column(main::entities::payment_transactions::Column::PaymentMethod)
        .column(main::entities::payment_transactions::Column::InvoiceUrl)
        .column(main::entities::payment_transactions::Column::Description)
        .column(main::entities::payment_transactions::Column::FailureReason)
        .column(main::entities::payment_transactions::Column::Metadata)
        .column(main::entities::payment_transactions::Column::CreatedAt)
        .column(main::entities::payment_transactions::Column::UpdatedAt)
        .column(main::entities::payment_transactions::Column::DeletedAt)
        .into_model::<PaymentTransactionsResultData, PaymentTenantResultData>()
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch payment transaction: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?
        .ok_or_else(|| {
            ApiResponse::new(404, json!({ "message": "Payment transaction not found" }))
        })?;

    let subscription = main::entities::subscriptions::Entity::find()
        .filter(main::entities::subscriptions::Column::TenantId.eq(payment_transaction.tenant_id))
        .order_by_desc(main::entities::subscriptions::Column::CreatedAt)
        .select_only()
        .column(main::entities::subscriptions::Column::Status)
        .into_model::<PaymentSubscriptionResultData>()
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch subscription: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "payment_transaction": {
                "pid": payment_transaction.pid,
                "tenant_id": payment_transaction.tenant_id,
                "tenant_pid": tenant.as_ref().map(|t| t.tenant_pid.clone()).unwrap_or_default(),
                "tenant_name": tenant.as_ref().map(|t| t.tenant_name.clone()).unwrap_or_default(),
                "subscription_id": payment_transaction.subscription_id,
                "subscription_status": subscription.as_ref().map(|s| s.status.clone()),
                "amount": payment_transaction.amount,
                "currency": payment_transaction.currency,
                "status": payment_transaction.status,
                "payment_method": payment_transaction.payment_method,
                "invoice_url": payment_transaction.invoice_url,
                "description": payment_transaction.description,
                "failure_reason": payment_transaction.failure_reason,
                "metadata": payment_transaction.metadata,
                "created_at": payment_transaction.created_at,
                "updated_at": payment_transaction.updated_at,
                "deleted_at": payment_transaction.deleted_at,
            },
            "message": "Payment transaction fetched successfully"
        }),
    ))
}

pub async fn show_by_tenant(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();

    let tenant_id = main::entities::tenants::Entity::find_by_pid(pid)
        .select_only()
        .column(main::entities::tenants::Column::Id)
        .into_tuple::<i32>()
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch tenant: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?
        .ok_or_else(|| ApiResponse::new(404, json!({ "message": "Tenant not found" })))?;

    let payment_transactions = main::entities::payment_transactions::Entity::find()
        .filter(main::entities::payment_transactions::Column::TenantId.eq(tenant_id))
        .filter(main::entities::payment_transactions::Column::DeletedAt.is_null())
        .order_by_desc(main::entities::payment_transactions::Column::CreatedAt)
        .all(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch payment transactions: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    let results: Vec<_> = payment_transactions
        .into_iter()
        .map(|p| {
            json!({
                "pid": p.pid,
                "amount": p.amount,
                "currency": p.currency,
                "status": p.status,
                "payment_method": p.payment_method,
                "invoice_url": p.invoice_url,
                "description": p.description,
                "failure_reason": p.failure_reason,
                "metadata": p.metadata,
                "created_at": p.created_at,
                "updated_at": p.updated_at,
            })
        })
        .collect();

    Ok(ApiResponse::new(
        200,
        json!({
            "payment_transactions": results,
            "message": "Payment transactions fetched successfully"
        }),
    ))
}

#[derive(Serialize, Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct PaymentTransactionStatusRequest {
    pub status: Option<String>,
}

impl PaymentTransactionStatusRequest {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if let Some(status) = &self.status {
            let status_lower = status.to_lowercase();
            if !["pending", "succeeded", "failed", "refunded"].contains(&status_lower.as_str()) {
                errors.insert(
                    "status".into(),
                    "Status must be one of: pending, succeeded, failed, refunded.".into(),
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

    pub fn get_status(&self) -> PaymentStatus {
        match self.status.as_ref().map(|s| s.to_lowercase()).as_deref() {
            Some("pending") => PaymentStatus::Pending,
            Some("succeeded") => PaymentStatus::Succeeded,
            Some("failed") => PaymentStatus::Failed,
            Some("refunded") => PaymentStatus::Refunded,
            _ => PaymentStatus::Pending,
        }
    }
}

pub async fn set_active_status(
    app_state: web::Data<AppState>,
    data: web::Json<PaymentTransactionStatusRequest>,
    path: web::Path<Uuid>,
) -> Result<ApiResponse, ApiResponse> {
    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    let pid = path.into_inner();

    let payment = main::entities::payment_transactions::Entity::find_by_pid(pid)
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch payment transaction: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?
        .ok_or_else(|| {
            ApiResponse::new(404, json!({ "message": "Payment transaction not found" }))
        })?;

    let mut update_model: main::entities::payment_transactions::ActiveModel =
        payment.to_owned().into();
    let mut changed = false;

    if data.get_status() != payment.status {
        update_model.status = Set(data.get_status());
        changed = true;
    }

    if !changed {
        return Err(ApiResponse::new(
            400,
            json!({ "message": "No updates were made because the data is unchanged.", }),
        ));
    }

    update_model.updated_at = Set(chrono::Utc::now().naive_utc());
    update_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update payment transaction: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": "Payment status updated successfully"
        }),
    ))
}
