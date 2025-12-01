use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("payment_status"))
                    .values([
                        Alias::new("pending"),
                        Alias::new("succeeded"),
                        Alias::new("failed"),
                        Alias::new("refunded"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("payment_method"))
                    .values([
                        Alias::new("card"),
                        Alias::new("mpesa"),
                        Alias::new("paypal"),
                        Alias::new("cash"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PaymentTransactions::Table)
                    .if_not_exists()
                    .col(pk_auto(PaymentTransactions::Id))
                    .col(
                        uuid_uniq(PaymentTransactions::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(PaymentTransactions::TenantId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-payment_transactions-tenant_id")
                            .from(PaymentTransactions::Table, PaymentTransactions::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(PaymentTransactions::SubscriptionId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-payment_transactions-subscription_id")
                            .from(
                                PaymentTransactions::Table,
                                PaymentTransactions::SubscriptionId,
                            )
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(decimal(PaymentTransactions::Amount).decimal_len(10, 2))
                    .col(
                        string(PaymentTransactions::Currency)
                            .string_len(3)
                            .default("KES"),
                    )
                    .col(
                        enumeration(
                            PaymentTransactions::Status,
                            Alias::new("payment_status"),
                            vec![
                                Alias::new("pending"),
                                Alias::new("succeeded"),
                                Alias::new("failed"),
                                Alias::new("refunded"),
                            ],
                        )
                        .default("pending"),
                    )
                    .col(enumeration(
                        PaymentTransactions::PaymentMethod,
                        Alias::new("payment_method"),
                        vec![
                            Alias::new("card"),
                            Alias::new("mpesa"),
                            Alias::new("paypal"),
                            Alias::new("cash"),
                        ],
                    ))
                    .col(text_null(PaymentTransactions::InvoiceUrl))
                    .col(text_null(PaymentTransactions::Description))
                    .col(text_null(PaymentTransactions::FailureReason))
                    .col(json_null(PaymentTransactions::Metadata))
                    .col(timestamp_null(PaymentTransactions::DeletedAt))
                    .col(
                        timestamp(PaymentTransactions::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(PaymentTransactions::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        let _idx_payment_transactions_tenant_id = Index::create()
            .name("idx_payment_transactions_tenant_id")
            .table(PaymentTransactions::Table)
            .col(PaymentTransactions::TenantId)
            .to_owned();

        let _idx_payment_transactions_status = Index::create()
            .name("idx_payment_transactions_status")
            .table(PaymentTransactions::Table)
            .col(PaymentTransactions::Status)
            .to_owned();

        let _idx_payment_transactions_created_at = Index::create()
            .name("idx_payment_transactions_created_at")
            .table(PaymentTransactions::Table)
            .col(PaymentTransactions::CreatedAt)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PaymentTransactions::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Alias::new("payment_status")).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Alias::new("payment_method")).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PaymentTransactions {
    Table,
    Id,
    Pid,
    TenantId,
    SubscriptionId,
    Amount,
    Currency,
    Status,
    PaymentMethod,
    InvoiceUrl,
    Description,
    FailureReason,
    Metadata,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Subscriptions {
    Table,
    Id,
}
