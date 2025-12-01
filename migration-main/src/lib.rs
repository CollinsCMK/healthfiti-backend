pub use sea_orm_migration::prelude::*;

// mod m20220101_000001_create_table;
mod m20251201_184153_create_tenants_table;
mod m20251201_190029_create_subscription_plans_table;
mod m20251201_191509_create_subscriptions_table;
mod m20251201_192310_create_payment_transactions_table;
mod m20251201_193439_create_billing_line_items_table;
mod m20251201_194543_create_features_table;
mod m20251201_194953_create_subscription_plan_features_table;
mod m20251201_195642_create_tenant_features_table;
mod m20251201_200245_create_feature_usage_logs_table;
mod m20251201_200850_create_feature_flags_table;
mod m20251201_202456_create_patients_table;
mod m20251201_203250_create_insurance_providers_table;
mod m20251201_203800_create_patient_insurance_table;
mod m20251201_203842_create_insurance_dependents_table;
mod m20251201_210232_create_global_system_logs_table;
mod m20251201_210712_create_usage_metrics_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20251201_184153_create_tenants_table::Migration),
            Box::new(m20251201_190029_create_subscription_plans_table::Migration),
            Box::new(m20251201_191509_create_subscriptions_table::Migration),
            Box::new(m20251201_192310_create_payment_transactions_table::Migration),
            Box::new(m20251201_193439_create_billing_line_items_table::Migration),
            Box::new(m20251201_194543_create_features_table::Migration),
            Box::new(m20251201_194953_create_subscription_plan_features_table::Migration),
            Box::new(m20251201_195642_create_tenant_features_table::Migration),
            Box::new(m20251201_200245_create_feature_usage_logs_table::Migration),
            Box::new(m20251201_200850_create_feature_flags_table::Migration),
            Box::new(m20251201_202456_create_patients_table::Migration),
            Box::new(m20251201_203250_create_insurance_providers_table::Migration),
            Box::new(m20251201_203800_create_patient_insurance_table::Migration),
            Box::new(m20251201_203842_create_insurance_dependents_table::Migration),
            Box::new(m20251201_210232_create_global_system_logs_table::Migration),
            Box::new(m20251201_210712_create_usage_metrics_table::Migration),
        ]
    }
}
