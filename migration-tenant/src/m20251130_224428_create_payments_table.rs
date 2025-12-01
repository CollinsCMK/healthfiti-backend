use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create enum for payment method
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("payment_method_enum"))
                    .values([
                        Alias::new("mpesa"),
                        Alias::new("airtel"),
                        Alias::new("card"),
                        Alias::new("paypal"),
                        Alias::new("cash"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Payments::Table)
                    .if_not_exists()
                    .col(pk_uuid(Payments::Id))
                    .col(
                        uuid_uniq(Payments::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(uuid(Payments::InvoiceId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-payments-invoice_id")
                            .from(Payments::Table, Payments::InvoiceId)
                            .to(BillingInvoices::Table, BillingInvoices::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(uuid_null(Payments::PaymentMethodId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-payments-payment_method_id")
                            .from(Payments::Table, Payments::PaymentMethodId)
                            .to(PaymentMethods::Table, PaymentMethods::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(decimal(Payments::Amount))
                    .col(enumeration(
                        Payments::Method,
                        Alias::new("payment_method_enum"),
                        vec![
                            Alias::new("mpesa"),
                            Alias::new("airtel"),
                            Alias::new("card"),
                            Alias::new("paypal"),
                            Alias::new("cash"),
                        ],
                    ))
                    .col(string_null(Payments::MpesaReceipt))
                    .col(string_null(Payments::TransactionReference))
                    .col(
                        timestamp(Payments::PaidAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(timestamp_null(Payments::DeletedAt))
                    .col(
                        timestamp(Payments::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(Payments::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Payments::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("payment_method_enum"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Payments {
    Table,
    Id,
    Pid,
    InvoiceId,
    PaymentMethodId,
    Amount,
    Method,
    MpesaReceipt,
    TransactionReference,
    PaidAt,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BillingInvoices {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum PaymentMethods {
    Table,
    Id,
}
