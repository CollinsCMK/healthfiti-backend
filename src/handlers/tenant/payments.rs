use std::collections::HashMap;

use crate::{
    db::main::{
        self,
        entities::sea_orm_active_enums::{
            BillingItemType, PaymentMethod, PaymentStatus, SubscriptionStatus,
        },
        migrations::sea_orm::{
            ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait,
            QueryFilter, QueryOrder, QuerySelect, Set,
        },
    }, emails::invoice::send_invoice_email, handlers::{
        admin::payments::{PaymentSubscriptionResultData, PaymentTransactionsResultData},
        tenant::subscriptions::compute_period_end,
    }, utils::{
        self,
        api_response::ApiResponse,
        app_state::AppState,
        jwt::get_tenant_id,
        mpesa::{MpesaClient, StkPushResponse},
        pagination::PaginationParams,
        permission::has_permission,
        validator_error::ValidationError,
    }
};
use actix_web::{HttpRequest, web};
use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc, Datelike};
use redis::AsyncCommands;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub async fn index(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationParams>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let (tenant_id, _, _) = get_tenant_id(&req, &app_state).await?;

    let mut stmt = main::entities::payment_transactions::Entity::find()
        .filter(main::entities::payment_transactions::Column::TenantId.eq(tenant_id));

    if !has_permission("view_archived_payment_transactions", &req).await? {
        stmt = stmt.filter(main::entities::payment_transactions::Column::DeletedAt.is_null());
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

    let payment_transaction = stmt
        .filter(main::entities::payment_transactions::Column::DeletedAt.is_null())
        .select_only()
        .column(main::entities::payment_transactions::Column::Pid)
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
        .into_model::<PaymentTransactionsResultData>()
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
            },
            "message": "Payment transaction fetched successfully"
        }),
    ))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdempotentResponse {
    pub status_code: u16,
    pub body: serde_json::Value,
    pub created_at: i64,
}

const IDEMPOTENCY_KEY_PREFIX: &str = "idempotency:payment:";
const IDEMPOTENCY_TTL_SECONDS: u64 = 86400;

// ============================================================================
// REQUEST/RESPONSE STRUCTURES
// ============================================================================

#[derive(Serialize, Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct PaymentTransactionCreateRequest {
    pub domain: String,
    pub subscription_id: i32,
    pub amount: Decimal,
    pub currency: String,
    pub payment_method: Option<String>,
    pub description: Option<String>,
    // Mpesa fields
    pub country_code: Option<String>,
    pub phone_number: Option<String>,
    // PayPal fields
    pub paypal_email: Option<String>,
    // Card / Stripe fields
    pub card_number: Option<String>,
    pub card_exp_month: Option<u32>,
    pub card_exp_year: Option<u32>,
    pub card_cvc: Option<String>,
}

impl PaymentTransactionCreateRequest {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = HashMap::new();

        if self.domain.trim().is_empty() {
            errors.insert("domain".into(), "Domain is required.".into());
        }

        if self.subscription_id <= 0 {
            errors.insert(
                "subscription_id".into(),
                "Subscription ID must be a positive integer.".into(),
            );
        }

        if self.amount <= Decimal::ZERO {
            errors.insert("amount".into(), "Amount must be greater than zero.".into());
        }

        if self.currency.trim().is_empty() {
            errors.insert("currency".into(), "Currency is required.".into());
        }

        let method = self.payment_method.as_deref().unwrap_or("").to_lowercase();
        if !["mpesa", "paypal", "card", "cash"].contains(&method.as_str()) {
            errors.insert(
                "payment_method".into(),
                "Payment method must be one of: mpesa, paypal, card, cash.".into(),
            );
        }

        match method.as_str() {
            "mpesa" => {
                if self.phone_number.is_none() {
                    errors.insert(
                        "phone_number".into(),
                        "Phone number is required for Mpesa.".into(),
                    );
                }
                if self.country_code.is_none() {
                    errors.insert(
                        "country_code".into(),
                        "Country code is required for Mpesa.".into(),
                    );
                }
            }
            "paypal" => {
                if self.paypal_email.is_none() {
                    errors.insert("paypal_email".into(), "PayPal email is required.".into());
                }
            }
            "card" => {
                if self.card_number.is_none() {
                    errors.insert("card_number".into(), "Card number is required.".into());
                }
                if self.card_exp_month.is_none() || self.card_exp_year.is_none() {
                    errors.insert(
                        "card_expiry".into(),
                        "Card expiry month & year are required.".into(),
                    );
                }
                if self.card_cvc.is_none() {
                    errors.insert("card_cvc".into(), "CVC is required.".into());
                }
            }
            _ => {}
        }

        if self.phone_number.is_some() && self.country_code.is_none() {
            errors.insert(
                "country_code".into(),
                "Country code is required when phone number is provided.".into(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { errors })
        }
    }

    pub fn get_payment_method(&self) -> PaymentMethod {
        match self.payment_method.as_deref().map(|s| s.to_lowercase()) {
            Some(s) if s == "mpesa" => PaymentMethod::Mpesa,
            Some(s) if s == "paypal" => PaymentMethod::Paypal,
            Some(s) if s == "card" => PaymentMethod::Card,
            Some(s) if s == "cash" => PaymentMethod::Cash,
            _ => PaymentMethod::Mpesa,
        }
    }
}

fn get_idempotency_redis_key(tenant_id: i32, idempotency_key: &str) -> String {
    format!("{}{}:{}", IDEMPOTENCY_KEY_PREFIX, tenant_id, idempotency_key)
}

async fn check_idempotency(
    app_state: &web::Data<AppState>,
    tenant_id: i32,
    idempotency_key: &str,
) -> Result<Option<IdempotentResponse>, ApiResponse> {
    let redis_key = get_idempotency_redis_key(tenant_id, idempotency_key);
    
    let mut conn = app_state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| {
            log::error!("Failed to get Redis connection: {}", err);
            ApiResponse::new(500, json!({ "message": "Redis connection failed" }))
        })?;

    let cached: Option<String> = conn.get(&redis_key).await.map_err(|err| {
        log::error!("Failed to get idempotency key from Redis: {}", err);
        ApiResponse::new(500, json!({ "message": "Failed to check idempotency" }))
    })?;

    if let Some(data) = cached {
        let response: IdempotentResponse = serde_json::from_str(&data).map_err(|err| {
            log::error!("Failed to deserialize cached response: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to parse cached response" }))
        })?;

        log::info!(
            "Idempotent request detected for key: {} (tenant: {})",
            idempotency_key,
            tenant_id
        );

        return Ok(Some(response));
    }

    Ok(None)
}

async fn store_idempotency_response(
    app_state: &web::Data<AppState>,
    tenant_id: i32,
    idempotency_key: &str,
    status_code: u16,
    body: serde_json::Value,
) -> Result<(), ApiResponse> {
    let redis_key = get_idempotency_redis_key(tenant_id, idempotency_key);
    
    let response = IdempotentResponse {
        status_code,
        body,
        created_at: Utc::now().timestamp(),
    };

    let serialized = serde_json::to_string(&response).map_err(|err| {
        log::error!("Failed to serialize response: {}", err);
        ApiResponse::new(500, json!({ "message": "Failed to cache response" }))
    })?;

    let mut conn = app_state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| {
            log::error!("Failed to get Redis connection: {}", err);
            ApiResponse::new(500, json!({ "message": "Redis connection failed" }))
        })?;

    let _: () = conn.set_ex(&redis_key, serialized, IDEMPOTENCY_TTL_SECONDS)
        .await
        .map_err(|err| {
            log::error!("Failed to store idempotency response: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to cache response" }))
        })?;

    log::info!(
        "Stored idempotency response for key: {} (tenant: {}), TTL: {}s",
        idempotency_key,
        tenant_id,
        IDEMPOTENCY_TTL_SECONDS
    );

    Ok(())
}

async fn delete_idempotency_key(
    app_state: &web::Data<AppState>,
    tenant_id: i32,
    idempotency_key: &str,
) -> Result<(), ApiResponse> {
    let redis_key = get_idempotency_redis_key(tenant_id, idempotency_key);
    
    let mut conn = app_state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| {
            log::error!("Failed to get Redis connection: {}", err);
            ApiResponse::new(500, json!({ "message": "Redis connection failed" }))
        })?;

    let _: () = conn.del(&redis_key).await.map_err(|err| {
        log::error!("Failed to delete idempotency key from Redis: {}", err);
        ApiResponse::new(500, json!({ "message": "Failed to delete idempotency key" }))
    })?;

    log::info!(
        "Deleted idempotency key: {} (tenant: {})",
        idempotency_key,
        tenant_id
    );

    Ok(())
}

#[derive(Debug)]
pub struct PaymentResult {
    pub status: String,
    pub failure_reason: Option<String>,
    pub response_json: serde_json::Value,
    pub provider_reference: Option<String>,
}

impl PaymentResult {
    pub fn success(response_json: serde_json::Value, provider_reference: Option<String>) -> Self {
        Self {
            status: "success".to_string(),
            failure_reason: None,
            response_json,
            provider_reference,
        }
    }

    pub fn failed(error_message: String, response_json: serde_json::Value) -> Self {
        Self {
            status: "failed".to_string(),
            failure_reason: Some(error_message),
            response_json,
            provider_reference: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.status == "success"
    }
}

// ============================================================================
// MPESA CALLBACK STRUCTURES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct MpesaCallbackRequest {
    #[serde(rename = "Body")]
    pub body: MpesaCallbackBody,
}

#[derive(Debug, Deserialize)]
pub struct MpesaCallbackBody {
    #[serde(rename = "stkCallback")]
    pub stk_callback: MpesaStkCallback,
}

#[derive(Debug, Deserialize)]
pub struct MpesaStkCallback {
    #[serde(rename = "MerchantRequestID")]
    pub merchant_request_id: String,
    #[serde(rename = "CheckoutRequestID")]
    pub checkout_request_id: String,
    #[serde(rename = "ResultCode")]
    pub result_code: i32,
    #[serde(rename = "ResultDesc")]
    pub result_desc: String,
    #[serde(rename = "CallbackMetadata")]
    pub callback_metadata: Option<MpesaCallbackMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct MpesaCallbackMetadata {
    #[serde(rename = "Item")]
    pub item: Vec<MpesaCallbackItem>,
}

#[derive(Debug, Deserialize)]
pub struct MpesaCallbackItem {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: serde_json::Value,
}

// ============================================================================
// PAYPAL WEBHOOK STRUCTURES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PayPalWebhook {
    pub id: String,
    pub event_type: String,
    pub resource: serde_json::Value,
}

// ============================================================================
// STRIPE WEBHOOK STRUCTURES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct StripeWebhook {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: StripeWebhookData,
}

#[derive(Debug, Deserialize)]
pub struct StripeWebhookData {
    pub object: serde_json::Value,
}

// ============================================================================
// PAYMENT PROCESSORS
// ============================================================================

#[async_trait]
pub trait PaymentProcessor {
    async fn process_payment(
        &self,
        data: &PaymentTransactionCreateRequest,
    ) -> Result<PaymentResult, String>;
}

pub struct MpesaProcessor;

#[async_trait]
impl PaymentProcessor for MpesaProcessor {
    async fn process_payment(
        &self,
        data: &PaymentTransactionCreateRequest,
    ) -> Result<PaymentResult, String> {
        let mpesa = MpesaClient::new();

        let timestamp = mpesa.get_timestamp();
        let password = mpesa.generate_password(&timestamp);
        let phone_number = format!(
            "{}{}",
            data.country_code.as_ref().unwrap(),
            data.phone_number.as_ref().unwrap()
        )
        .replace("+", "");

        let payload = json!({
            "BusinessShortCode": (utils::constants::MPESA_SHORTCODE).clone(),
            "Password": password,
            "Timestamp": timestamp,
            "TransactionType": (utils::constants::MPESA_TRANSACTION_TYPE).clone(),
            "Amount": data.amount,
            "PartyA": phone_number,
            "PartyB": (utils::constants::MPESA_SHORTCODE).clone(),
            "PhoneNumber": phone_number,
            "CallBackURL": (utils::constants::MPESA_CALLBACK_URL).clone(),
            "AccountReference": format!("Subscription {}", data.subscription_id),
            "TransactionDesc": data.description.as_ref().unwrap_or(&"Payment for subscription".to_string()),
        });

        let res = mpesa.stk_push(&payload).await.map_err(|err| {
            log::error!("Mpesa STK Push failed: {}", err);
            format!("Mpesa STK Push failed: {}", err)
        })?;

        let result = match res {
            StkPushResponse::Success {
                ResponseCode,
                ResponseDescription,
                MerchantRequestID,
                CheckoutRequestID,
                ResultCode,
                ResultDesc,
            } => PaymentResult::success(
                json!({
                    "message": ResponseDescription,
                    "response_code": ResponseCode,
                    "merchant_request_id": MerchantRequestID,
                    "checkout_request_id": CheckoutRequestID,
                    "result_code": ResultCode,
                    "result_desc": ResultDesc,
                }),
                Some(CheckoutRequestID),
            ),

            StkPushResponse::Error {
                requestId,
                errorCode,
                errorMessage,
            } => {
                log::error!("Mpesa STK Error [{}]: {}", errorCode, errorMessage);
                PaymentResult::failed(
                    format!("{}: {}", errorCode, errorMessage),
                    json!({
                        "request_id": requestId,
                        "error_code": errorCode,
                        "error_message": errorMessage,
                    }),
                )
            }
        };

        Ok(result)
    }
}

pub struct PayPalProcessor;

#[async_trait]
impl PaymentProcessor for PayPalProcessor {
    async fn process_payment(
        &self,
        data: &PaymentTransactionCreateRequest,
    ) -> Result<PaymentResult, String> {
        log::info!(
            "Processing PayPal payment for email: {:?}",
            data.paypal_email
        );

        // TODO: Implement actual PayPal API integration
        // For now, return a dummy response
        let order_id = format!("PAYPAL_ORDER_{}", Utc::now().timestamp());

        Ok(PaymentResult::success(
            json!({
                "message": "PayPal payment initiated",
                "paypal_email": data.paypal_email,
                "order_id": order_id.clone(),
                "approval_url": format!("https://www.paypal.com/checkoutnow?token={}", order_id),
                "note": "This is a placeholder - implement actual PayPal SDK integration"
            }),
            Some(order_id),
        ))
    }
}

pub struct CardProcessor;

#[async_trait]
impl PaymentProcessor for CardProcessor {
    async fn process_payment(
        &self,
        data: &PaymentTransactionCreateRequest,
    ) -> Result<PaymentResult, String> {
        log::info!(
            "Processing card payment for subscription: {}",
            data.subscription_id
        );

        // TODO: Implement actual Stripe API integration
        let payment_intent_id = format!("pi_{}", Utc::now().timestamp());

        Ok(PaymentResult::success(
            json!({
                "message": "Card payment processed",
                "payment_intent_id": payment_intent_id.clone(),
                "card_last4": "****",
                "status": "requires_confirmation",
                "client_secret": format!("{}_secret_DUMMY", payment_intent_id),
                "note": "This is a placeholder - implement actual Stripe SDK integration"
            }),
            Some(payment_intent_id),
        ))
    }
}

pub struct CashProcessor;

#[async_trait]
impl PaymentProcessor for CashProcessor {
    async fn process_payment(
        &self,
        data: &PaymentTransactionCreateRequest,
    ) -> Result<PaymentResult, String> {
        log::info!(
            "Recording cash payment for subscription: {}",
            data.subscription_id
        );

        let reference = format!("CASH-{}-{}", data.subscription_id, Utc::now().timestamp());

        Ok(PaymentResult::success(
            json!({
                "message": "Cash payment recorded",
                "reference_number": reference.clone(),
                "status": "pending_confirmation",
                "note": "Cash payment pending manual confirmation"
            }),
            Some(reference),
        ))
    }
}

pub fn get_payment_processor(method: &PaymentMethod) -> Box<dyn PaymentProcessor> {
    match method {
        PaymentMethod::Mpesa => Box::new(MpesaProcessor),
        PaymentMethod::Paypal => Box::new(PayPalProcessor),
        PaymentMethod::Card => Box::new(CardProcessor),
        PaymentMethod::Cash => Box::new(CashProcessor),
    }
}

// ============================================================================
// SUBSCRIPTION HELPER
// ============================================================================

pub async fn create_subscription(
    app_state: &web::Data<AppState>,
    plan_id: i32,
    tenant_id: i32,
) -> Result<(NaiveDateTime, NaiveDateTime), ApiResponse> {
    let plan = main::entities::subscription_plans::Entity::find_by_id(plan_id)
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
    let end = compute_period_end(start, &plan.billing_cycle, None);

    let existing_sub = main::entities::subscriptions::Entity::find()
        .filter(main::entities::subscriptions::Column::TenantId.eq(tenant_id))
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch subscription: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    if let Some(existing) = existing_sub.clone() {
        if existing.status == SubscriptionStatus::Active
            || existing.status == SubscriptionStatus::Trial
        {
            return Err(ApiResponse::new(
                409,
                json!({ "message": "Tenant already has an active subscription" }),
            ));
        }
    }

    let give_trial = existing_sub.is_none() && plan.trial_days > 0;
    let status = if give_trial {
        SubscriptionStatus::Trial
    } else {
        SubscriptionStatus::PendingPayment
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

    Ok((start, end))
}

// ============================================================================
// PAYMENT CREATION ENDPOINT
// ============================================================================

pub async fn create(
    app_state: web::Data<AppState>,
    data: web::Json<PaymentTransactionCreateRequest>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let (tenant_id, _, _) = get_tenant_id(&req, &app_state).await?;

    if let Err(err) = data.validate() {
        return Err(ApiResponse::new(400, json!(err)));
    }

    let new_idempotency_key = Uuid::new_v4().to_string();

    if let Some(cached_response) = check_idempotency(&app_state, tenant_id, &new_idempotency_key).await? {
        log::info!(
            "Returning cached response for idempotency key: {}",
            new_idempotency_key
        );
        return Ok(ApiResponse::new(
            cached_response.status_code,
            cached_response.body,
        ));
    }

    let payment_method = data.get_payment_method();
    let processor = get_payment_processor(&payment_method);

    let result = processor.process_payment(&data).await.map_err(|err| {
        log::error!("Payment processing failed: {}", err);
        ApiResponse::new(500, json!({ "message": err }))
    })?;

    if !result.is_success() {
        let error_response = json!({
            "message": format!("{:?} payment failed", payment_method),
            "details": result.response_json,
        });
        
        let _ = store_idempotency_response(
            &app_state,
            tenant_id,
            &new_idempotency_key,
            500,
            error_response.clone(),
        )
        .await;

        if should_allow_retry(&result.failure_reason) {
            log::info!("Deleting idempotency key to allow retry");
            let _ = delete_idempotency_key(&app_state, tenant_id, &new_idempotency_key).await;
        }

        return Err(ApiResponse::new(500, error_response));
    }

    let (start, end) = create_subscription(&app_state, data.subscription_id, tenant_id).await?;

    let billing_item = main::entities::billing_line_items::ActiveModel {
        tenant_id: Set(tenant_id),
        subscription_id: Set(data.subscription_id),
        billing_period_start: Set(start.into()),
        billing_period_end: Set(end.into()),
        item_type: Set(BillingItemType::SubscriptionFee),
        description: Set("Subscription fee".to_string()),
        unit_price: Set(data.amount),
        total_amount: Set(data.amount),
        metadata: Set(Some(result.response_json.clone())),
        ..Default::default()
    }
        .insert(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to create billing line item: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    let tenant = main::entities::tenants::Entity::find_by_id(tenant_id)
        .filter(main::entities::tenants::Column::DeletedAt.is_null())
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch tenant: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to fetch tenant" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Tenant not found" }),
        ))?;

    let invoice_number = format!("INV-{}-{:06}", Utc::now().year(), billing_item.pid);

    let tenant_email = tenant
        .contact_email
        .clone()
        .ok_or_else(|| ApiResponse::new(400, json!({ "message": "Tenant email not set" })))?;

    let start_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();

    let items = vec![("Subscription fee", 1, data.amount)];

    let invoice_url = send_invoice_email(
        tenant_email,
        &tenant.name,
        &invoice_number,
        &start_str,
        &end_str,
        items,
        &data.amount,
        &app_state,
        &req,
        Some(data.domain.clone()),
        None,
        None
    )
    .await
    .map_err(|err| {
        log::error!("Failed to send invoice email: {}", err);
        ApiResponse::new(500, json!({ "message": "Failed to send invoice email" }))
    })?;

    let transaction = main::entities::payment_transactions::ActiveModel {
        tenant_id: Set(tenant_id),
        subscription_id: Set(data.subscription_id),
        amount: Set(data.amount),
        currency: Set(data.currency.clone()),
        status: Set(PaymentStatus::Pending),
        payment_method: Set(payment_method.clone()),
        invoice_url: Set(Some(invoice_url)),
        description: Set(data.description.clone()),
        failure_reason: Set(result.failure_reason.clone()),
        metadata: Set(Some(json!({
            "idempotency_key": new_idempotency_key,
            "payment_result": result.response_json,
            "provider_reference": result.provider_reference,
        }))),
        ..Default::default()
    }
        .insert(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to create payment transaction: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    let success_response = json!({
        "message": "Payment initiated successfully",
        "status": "pending",
        "transaction_id": transaction.id,
        "provider_reference": result.provider_reference,
        "details": result.response_json,
        "idempotency_key": new_idempotency_key,
    });

    store_idempotency_response(
        &app_state,
        tenant_id,
        &new_idempotency_key,
        200,
        success_response.clone(),
    )
    .await?;

    Ok(ApiResponse::new(200, success_response))
}

fn should_allow_retry(failure_reason: &Option<String>) -> bool {
    if let Some(reason) = failure_reason {
        let reason_lower = reason.to_lowercase();
        reason_lower.contains("timeout") ||
        reason_lower.contains("network") ||
        reason_lower.contains("unavailable") ||
        reason_lower.contains("503")
    } else {
        false
    }
}

pub async fn retry_payment(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    let pid = path.into_inner();
    let (tenant_id, _, _) = get_tenant_id(&req, &app_state).await?;

    // Find the failed transaction
    let transaction = main::entities::payment_transactions::Entity::find_by_pid(pid)
        .filter(main::entities::payment_transactions::Column::TenantId.eq(tenant_id))
        .filter(main::entities::payment_transactions::Column::Status.eq(PaymentStatus::Failed))
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find transaction: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?
        .ok_or_else(|| {
            ApiResponse::new(404, json!({ "message": "Failed transaction not found" }))
        })?;

    // Extract idempotency key from metadata
    let idempotency_key = transaction
        .metadata
        .as_ref()
        .and_then(|m| m["idempotency_key"].as_str())
        .ok_or_else(|| {
            ApiResponse::new(400, json!({ "message": "Idempotency key not found in metadata" }))
        })?;

    delete_idempotency_key(&app_state, tenant_id, idempotency_key).await?;

    let new_idempotency_key = uuid::Uuid::new_v4().to_string();

    let mut metadata = transaction.metadata.clone().unwrap_or_default();
    metadata["idempotency_key"] = serde_json::Value::String(new_idempotency_key.clone());

    let mut active_model: main::entities::payment_transactions::ActiveModel = transaction.to_owned().into();
    active_model.metadata = Set(Some(metadata));
    active_model.updated_at = Set(Utc::now().naive_utc());
    active_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update transaction metadata: {}", err);
            ApiResponse::new(500, json!({ "message": err.to_string() }))
        })?;

    store_idempotency_response(
        &app_state,
        tenant_id,
        &new_idempotency_key,
        200,
        json!({ "message": "Retry initiated" }),
    )
    .await?;

    Ok(ApiResponse::new(
        200,
        json!({
            "message": "Payment retry enabled. Use the new idempotency key for the next attempt.",
            "idempotency_key": new_idempotency_key,
        }),
    ))
}

// ============================================================================
// MPESA CALLBACK HANDLER
// ============================================================================

pub async fn mpesa_callback(
    app_state: web::Data<AppState>,
    payload: web::Json<MpesaCallbackRequest>,
) -> Result<ApiResponse, ApiResponse> {
    log::info!("M-Pesa callback received: {:?}", payload);

    let callback = &payload.body.stk_callback;
    let checkout_request_id = &callback.checkout_request_id;

    // Find the pending transaction by checkout_request_id in metadata
    let transaction = main::entities::payment_transactions::Entity::find()
        .filter(
            main::entities::payment_transactions::Column::Metadata.contains(checkout_request_id),
        )
        .filter(main::entities::payment_transactions::Column::Status.eq(PaymentStatus::Pending))
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find transaction: {}", err);
            ApiResponse::new(500, json!({ "message": "Transaction lookup failed" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Transaction not found" }),
        ))?;

    let (new_status, failure_reason) = if callback.result_code == 0 {
        (PaymentStatus::Succeeded, None)
    } else {
        (PaymentStatus::Failed, Some(callback.result_desc.clone()))
    };

    // Extract payment details from callback metadata
    let mut mpesa_receipt = None;
    let mut phone_number = None;
    let mut amount = None;

    if let Some(metadata) = &callback.callback_metadata {
        for item in &metadata.item {
            match item.name.as_str() {
                "MpesaReceiptNumber" => {
                    mpesa_receipt = item.value.as_str().map(|s| s.to_string());
                }
                "PhoneNumber" => {
                    phone_number = item.value.as_u64().map(|n| n.to_string());
                }
                "Amount" => {
                    amount = item.value.as_f64();
                }
                _ => {}
            }
        }
    }

    // Update transaction
    let mut active_model: main::entities::payment_transactions::ActiveModel =
        transaction.to_owned().into();
    active_model.status = Set(new_status.clone());
    active_model.failure_reason = Set(failure_reason);

    // Merge callback data into metadata
    let mut existing_metadata = transaction
        .metadata
        .as_ref()
        .and_then(|m| m.as_object().cloned())
        .unwrap_or_default();

    existing_metadata.insert(
        "mpesa_callback".to_string(),
        json!({
            "merchant_request_id": callback.merchant_request_id,
            "checkout_request_id": callback.checkout_request_id,
            "result_code": callback.result_code,
            "result_desc": callback.result_desc,
            "receipt_number": mpesa_receipt,
            "phone_number": phone_number,
            "amount": amount,
            "callback_time": Utc::now().to_rfc3339(),
        }),
    );

    active_model.metadata = Set(Some(serde_json::Value::Object(existing_metadata)));
    active_model.updated_at = Set(Utc::now().naive_utc());
    let updated = active_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update transaction: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to update transaction" }))
        })?;

    if new_status == PaymentStatus::Succeeded {
        activate_subscription(&app_state, updated.subscription_id, updated.tenant_id).await?;

        let idempotency_key = transaction
            .metadata
            .as_ref()
            .and_then(|m| m["idempotency_key"].as_str())
            .ok_or_else(|| {
                ApiResponse::new(400, json!({ "message": "Idempotency key not found in metadata" }))
            })?;

        delete_idempotency_key(&app_state, transaction.tenant_id, idempotency_key).await?;
    }

    Ok(ApiResponse::new(
        200,
        json!({ "ResultCode": 0, "ResultDesc": "Callback processed successfully" }),
    ))
}

// ============================================================================
// PAYPAL WEBHOOK HANDLER
// ============================================================================

pub async fn paypal_webhook(
    app_state: web::Data<AppState>,
    payload: web::Json<PayPalWebhook>,
    _req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    log::info!("PayPal webhook received: {:?}", payload);

    // TODO: Verify webhook signature
    // verify_paypal_webhook(&payload, &req)?;

    match payload.event_type.as_str() {
        "PAYMENT.CAPTURE.COMPLETED" => {
            let order_id = payload.resource["id"].as_str().unwrap_or("");

            let transaction = find_transaction_by_metadata(&app_state, order_id).await?;

            update_transaction_status(
                &app_state,
                transaction,
                PaymentStatus::Succeeded,
                None,
                json!({
                    "event_type": payload.event_type,
                    "event_id": payload.id,
                    "resource": payload.resource,
                    "callback_time": Utc::now().to_rfc3339(),
                }),
            )
            .await?;
        }
        "PAYMENT.CAPTURE.DENIED" | "PAYMENT.CAPTURE.DECLINED" => {
            let order_id = payload.resource["id"].as_str().unwrap_or("");

            let transaction = find_transaction_by_metadata(&app_state, order_id).await?;

            update_transaction_status(
                &app_state,
                transaction,
                PaymentStatus::Failed,
                Some("Payment declined by PayPal".to_string()),
                json!({
                    "event_type": payload.event_type,
                    "event_id": payload.id,
                    "resource": payload.resource,
                    "callback_time": Utc::now().to_rfc3339(),
                }),
            )
            .await?;
        }
        _ => {
            log::info!("Unhandled PayPal event type: {}", payload.event_type);
        }
    }

    Ok(ApiResponse::new(
        200,
        json!({ "message": "Webhook processed" }),
    ))
}

// ============================================================================
// STRIPE WEBHOOK HANDLER
// ============================================================================

pub async fn stripe_webhook(
    app_state: web::Data<AppState>,
    payload: web::Json<StripeWebhook>,
    _req: HttpRequest,
) -> Result<ApiResponse, ApiResponse> {
    log::info!("Stripe webhook received: {:?}", payload);

    // TODO: Verify webhook signature
    // verify_stripe_webhook(&payload, &req)?;

    match payload.event_type.as_str() {
        "payment_intent.succeeded" => {
            let payment_intent_id = payload.data.object["id"].as_str().unwrap_or("");

            let transaction = find_transaction_by_metadata(&app_state, payment_intent_id).await?;

            update_transaction_status(
                &app_state,
                transaction,
                PaymentStatus::Succeeded,
                None,
                json!({
                    "event_type": payload.event_type,
                    "event_id": payload.id,
                    "object": payload.data.object,
                    "callback_time": Utc::now().to_rfc3339(),
                }),
            )
            .await?;
        }
        "payment_intent.payment_failed" => {
            let payment_intent_id = payload.data.object["id"].as_str().unwrap_or("");
            let error_message = payload.data.object["last_payment_error"]["message"]
                .as_str()
                .unwrap_or("Payment failed");

            let transaction = find_transaction_by_metadata(&app_state, payment_intent_id).await?;

            update_transaction_status(
                &app_state,
                transaction,
                PaymentStatus::Failed,
                Some(error_message.to_string()),
                json!({
                    "event_type": payload.event_type,
                    "event_id": payload.id,
                    "object": payload.data.object,
                    "callback_time": Utc::now().to_rfc3339(),
                }),
            )
            .await?;
        }
        _ => {
            log::info!("Unhandled Stripe event type: {}", payload.event_type);
        }
    }

    Ok(ApiResponse::new(200, json!({ "received": true })))
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async fn find_transaction_by_metadata(
    app_state: &web::Data<AppState>,
    reference: &str,
) -> Result<main::entities::payment_transactions::Model, ApiResponse> {
    main::entities::payment_transactions::Entity::find()
        .filter(main::entities::payment_transactions::Column::Metadata.contains(reference))
        .filter(main::entities::payment_transactions::Column::Status.eq(PaymentStatus::Pending))
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find transaction: {}", err);
            ApiResponse::new(500, json!({ "message": "Transaction lookup failed" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Transaction not found" }),
        ))
}

async fn update_transaction_status(
    app_state: &web::Data<AppState>,
    transaction: main::entities::payment_transactions::Model,
    status: PaymentStatus,
    failure_reason: Option<String>,
    callback_data: serde_json::Value,
) -> Result<(), ApiResponse> {
    let mut active_model: main::entities::payment_transactions::ActiveModel =
        transaction.to_owned().into();
    active_model.status = Set(status.clone());
    active_model.failure_reason = Set(failure_reason);

    let mut existing_metadata = transaction
        .metadata
        .as_ref()
        .and_then(|m| m.as_object().cloned())
        .unwrap_or_default();

    existing_metadata.insert("callback_data".to_string(), callback_data);
    active_model.metadata = Set(Some(serde_json::Value::Object(existing_metadata)));
    active_model.updated_at = Set(Utc::now().naive_utc());
    active_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to update transaction: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to update transaction" }))
        })?;

    if status == PaymentStatus::Succeeded {
        activate_subscription(
            app_state,
            transaction.subscription_id,
            transaction.tenant_id,
        )
        .await?;
    }

    Ok(())
}

async fn activate_subscription(
    app_state: &web::Data<AppState>,
    subscription_id: i32,
    tenant_id: i32,
) -> Result<(), ApiResponse> {
    let subscription = main::entities::subscriptions::Entity::find()
        .filter(main::entities::subscriptions::Column::Id.eq(subscription_id))
        .filter(main::entities::subscriptions::Column::TenantId.eq(tenant_id))
        .one(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to find subscription: {}", err);
            ApiResponse::new(500, json!({ "message": "Subscription lookup failed" }))
        })?
        .ok_or(ApiResponse::new(
            404,
            json!({ "message": "Subscription not found" }),
        ))?;

    let mut active_model: main::entities::subscriptions::ActiveModel = subscription.into();
    active_model.status = Set(SubscriptionStatus::Active);
    active_model.updated_at = Set(Utc::now().naive_utc());
    active_model
        .update(&app_state.main_db)
        .await
        .map_err(|err| {
            log::error!("Failed to activate subscription: {}", err);
            ApiResponse::new(500, json!({ "message": "Failed to activate subscription" }))
        })?;

    log::info!(
        "Subscription {} activated for tenant {}",
        subscription_id,
        tenant_id
    );

    Ok(())
}
