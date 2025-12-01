use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create enum for invoice status
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("billing_invoice_status"))
                    .values([
                        Alias::new("pending"),
                        Alias::new("paid"),
                        Alias::new("cancelled"),
                        Alias::new("overdue"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(BillingInvoices::Table)
                    .if_not_exists()
                    .col(pk_uuid(BillingInvoices::Id))
                    .col(
                        uuid_uniq(BillingInvoices::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(uuid(BillingInvoices::MainPatientId))
                    .col(text(BillingInvoices::Description))
                    .col(decimal(BillingInvoices::TotalAmount))
                    .col(
                        enumeration(
                            BillingInvoices::Status,
                            Alias::new("billing_invoice_status"),
                            vec![
                                Alias::new("pending"),
                                Alias::new("paid"),
                                Alias::new("cancelled"),
                                Alias::new("overdue"),
                            ],
                        )
                        .default("pending"),
                    )
                    .col(date_null(BillingInvoices::DueDate))
                    .col(timestamp_null(BillingInvoices::DeletedAt))
                    .col(
                        timestamp(BillingInvoices::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(BillingInvoices::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BillingInvoices::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("billing_invoice_status"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum BillingInvoices {
    Table,
    Id,
    Pid,
    MainPatientId,
    Description,
    TotalAmount,
    Status,
    DueDate,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}
