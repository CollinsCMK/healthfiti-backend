pub use sea_orm_migration::prelude::*;

// mod m20220101_000001_create_table;
mod m20251124_131907_create_users_table;
mod m20251124_132808_create_facilities_table;
mod m20251124_135912_create_facility_staff_table;
mod m20251124_140316_create_facility_reviews_table;
mod m20251124_141101_create_service_categories_table;
mod m20251124_141336_create_services_table;
mod m20251124_141833_create_service_reviews_table;
mod m20251124_142157_create_facility_services_table;
mod m20251130_223852_create_insurance_claims_table;
mod m20251130_224209_create_billing_invoices_table;
mod m20251130_224415_create_invoice_items_table;
mod m20251130_224428_create_payments_table;
mod m20251130_224606_create_refund_requests_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20251124_131907_create_users_table::Migration),
            Box::new(m20251124_132808_create_facilities_table::Migration),
            Box::new(m20251124_135912_create_facility_staff_table::Migration),
            Box::new(m20251124_140316_create_facility_reviews_table::Migration),
            Box::new(m20251124_141101_create_service_categories_table::Migration),
            Box::new(m20251124_141336_create_services_table::Migration),
            Box::new(m20251124_141833_create_service_reviews_table::Migration),
            Box::new(m20251124_142157_create_facility_services_table::Migration),
            Box::new(m20251130_223852_create_insurance_claims_table::Migration),
            Box::new(m20251130_224209_create_billing_invoices_table::Migration),
            Box::new(m20251130_224415_create_invoice_items_table::Migration),
            Box::new(m20251130_224428_create_payments_table::Migration),
            Box::new(m20251130_224606_create_refund_requests_table::Migration),
        ]
    }
}
