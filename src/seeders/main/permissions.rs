use reqwest::Method;
use serde::Deserialize;
use serde_json::json;

use crate::utils::{api_response::ApiResponse, http_client::ApiClient};

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct PermissionResponse {
    pub message: String,
}

pub async fn seed_permissions() -> Result<ApiResponse, ApiResponse> {
    let api = ApiClient::new();

    log::info!("SSO Base URL: {}", api.base_url);

    // name (snake_case), description, module
    let default_permissions = vec![
        // Patient Insurances
        (
            "view_all_patient_insurances",
            "Allows the user to view all patient insurance records in the system",
            "Patient Insurances",
        ),
        (
            "view_patient_insurance",
            "Allows the user to view a specific patient insurance record",
            "Patient Insurances",
        ),
        (
            "create_patient_insurance",
            "Allows the user to create a new patient insurance record",
            "Patient Insurances",
        ),
        (
            "update_patient_insurance",
            "Allows the user to update an existing patient insurance record",
            "Patient Insurances",
        ),
        (
            "update_primary_patient_insurance",
            "Allows the user to set or update whether a patient insurance record is primary",
            "Patient Insurances",
        ),
        (
            "delete_patient_insurance",
            "Permanently deletes a patient insurance record",
            "Patient Insurances",
        ),
        (
            "view_archived_patient_insurances",
            "Allows the user to view archived/soft-deleted patient insurance records",
            "Patient Insurances",
        ),
        (
            "soft_delete_patient_insurance",
            "Allows the user to soft-delete a patient insurance record",
            "Patient Insurances",
        ),
        (
            "restore_patient_insurance",
            "Allows the user to restore a soft-deleted patient insurance record",
            "Patient Insurances",
        ),
        // Subscription Plans
        (
            "view_all_subscription_plans",
            "Allows the user to view all subscription plans in the system",
            "Subscription Plans",
        ),
        (
            "view_subscription_plan",
            "Allows the user to view a specific subscription plan",
            "Subscription Plans",
        ),
        (
            "create_subscription_plan",
            "Allows the user to create a new subscription plan",
            "Subscription Plans",
        ),
        (
            "update_subscription_plan",
            "Allows the user to update an existing subscription plan",
            "Subscription Plans",
        ),
        (
            "activate_or_deactivate_subscription_plan",
            "Allows the user to activate or deactivate a subscription plan",
            "Subscription Plans",
        ),
        (
            "delete_subscription_plan",
            "Permanently deletes a subscription plan",
            "Subscription Plans",
        ),
        (
            "view_archived_subscription_plans",
            "Allows the user to view archived/soft-deleted subscription plans",
            "Subscription Plans",
        ),
        (
            "soft_delete_subscription_plan",
            "Allows the user to soft-delete a subscription plan",
            "Subscription Plans",
        ),
        (
            "restore_subscription_plan",
            "Allows the user to restore a soft-deleted subscription plan",
            "Subscription Plans",
        ),
        // Payment Transactions
        (
            "view_all_payment_transactions",
            "Allows the user to view all payment transactions in the system",
            "Payment Transactions",
        ),
        (
            "view_tenant_payment_transaction",
            "Allows the user to view a payment transaction for a specific tenant",
            "Payment Transactions",
        ),
        (
            "view_payment_transaction",
            "Allows the user to view a specific payment transaction",
            "Payment Transactions",
        ),
        (
            "create_payment_transaction",
            "Allows the user to create a new payment transaction",
            "Payment Transactions",
        ),
        (
            "retry_payment_transaction",
            "Allows the user to retry a failed payment transaction",
            "Payment Transactions",
        ),
        (
            "activate_or_deactivate_payment_transaction",
            "Allows the user to activate or deactivate a payment transaction",
            "Payment Transactions",
        ),
        (
            "delete_payment_transaction",
            "Permanently deletes a payment transaction",
            "Payment Transactions",
        ),
        (
            "view_archived_payment_transactions",
            "Allows the user to view archived/soft-deleted payment transactions",
            "Payment Transactions",
        ),
        (
            "soft_delete_payment_transaction",
            "Allows the user to soft-delete a payment transaction",
            "Payment Transactions",
        ),
        (
            "restore_payment_transaction",
            "Allows the user to restore a soft-deleted payment transaction",
            "Payment Transactions",
        ),
    ];

    // Convert default_permissions to Vec of objects expected by /create API
    let permissions_json: Vec<_> = default_permissions
        .into_iter()
        .map(|(name, description, module)| {
            json!({
                "name": name,
                "description": description,
                "module": module
            })
        })
        .collect();

    let request_json = json!(permissions_json);

    log::info!("Calling SSO API: {}/permissions/create", api.base_url);
    log::debug!("Request payload: {:?}", request_json);

    let response: PermissionResponse = api
        .call_with_secret("permissions/create", Some(&request_json), Method::POST)
        .await
        .map_err(|err| {
            log::error!("Permissions API error: {}", err);
            ApiResponse::new(
                500,
                json!({
                    "message": "Failed to seed permissions"
                }),
            )
        })?;

    Ok(ApiResponse::new(
        200,
        json!({ "message": response.message }),
    ))
}
