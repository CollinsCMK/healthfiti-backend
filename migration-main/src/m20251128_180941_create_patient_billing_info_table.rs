use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    schema::*,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("patient_billing_cycle"))
                    .values([
                        Alias::new("weekly"),
                        Alias::new("monthly"),
                        Alias::new("quarterly"),
                        Alias::new("yearly"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PatientBilling::Table)
                    .if_not_exists()
                    .col(pk_auto(PatientBilling::Id))
                    .col(
                        uuid_uniq(PatientBilling::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer_uniq(PatientBilling::PatientId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-patient_billing-patient_id")
                            .from(PatientBilling::Table, PatientBilling::PatientId)
                            .to(Patients::Table, Patients::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(enumeration(
                        PatientBilling::BillingCycle,
                        Alias::new("patient_billing_cycle"),
                        vec![
                            Alias::new("weekly"),
                            Alias::new("monthly"),
                            Alias::new("quarterly"),
                            Alias::new("yearly"),
                        ],
                    ))
                    .col(string_null(PatientBilling::PreferredPaymentMethod))
                    .col(boolean(PatientBilling::AutoPaymentEnabled).default(false))
                    .col(timestamp_null(PatientBilling::DeletedAt))
                    .col(
                        timestamp(PatientBilling::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(PatientBilling::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PatientBilling::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("patient_billing_cycle"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum PatientBilling {
    Table,
    Id,
    Pid,
    PatientId,
    BillingCycle,
    PreferredPaymentMethod,
    AutoPaymentEnabled,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Patients {
    Table,
    Id,
}
