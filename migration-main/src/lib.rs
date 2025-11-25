pub use sea_orm_migration::prelude::*;

// mod m20220101_000001_create_table;
mod m20251123_235006_create_patients_table;
mod m20251124_001103_create_features_table;
mod m20251124_001203_create_subscription_plans_table;
mod m20251124_001220_create_subscription_plan_features_table;
mod m20251124_001437_create_tenants_table;
mod m20251124_002548_create_tenant_features_table;
mod m20251124_002621_create_tenant_subscriptions_table;
mod m20251124_125559_create_patient_access_permissions_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20251123_235006_create_patients_table::Migration),
            Box::new(m20251124_001103_create_features_table::Migration),
            Box::new(m20251124_001203_create_subscription_plans_table::Migration),
            Box::new(m20251124_001220_create_subscription_plan_features_table::Migration),
            Box::new(m20251124_001437_create_tenants_table::Migration),
            Box::new(m20251124_002548_create_tenant_features_table::Migration),
            Box::new(m20251124_002621_create_tenant_subscriptions_table::Migration),
            Box::new(m20251124_125559_create_patient_access_permissions_table::Migration),
        ]
    }
}
