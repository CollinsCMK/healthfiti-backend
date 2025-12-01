use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create enum for refund status
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("refund_request_status"))
                    .values([
                        Alias::new("pending"),
                        Alias::new("approved"),
                        Alias::new("rejected"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RefundRequests::Table)
                    .if_not_exists()
                    .col(pk_uuid(RefundRequests::Id))
                    .col(
                        uuid_uniq(RefundRequests::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(RefundRequests::PaymentId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-refund_requests-payment_id")
                            .from(RefundRequests::Table, RefundRequests::PaymentId)
                            .to(PaymentMethods::Table, PaymentMethods::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(text_null(RefundRequests::Reason))
                    .col(decimal_null(RefundRequests::Amount))
                    .col(
                        enumeration(
                            RefundRequests::Status,
                            Alias::new("refund_request_status"),
                            vec![
                                Alias::new("pending"),
                                Alias::new("approved"),
                                Alias::new("rejected"),
                            ],
                        )
                        .default("pending"),
                    )
                    .col(timestamp_null(RefundRequests::DeletedAt))
                    .col(
                        timestamp(RefundRequests::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(RefundRequests::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RefundRequests::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("refund_request_status"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum RefundRequests {
    Table,
    Id,
    Pid,
    PaymentId,
    Reason,
    Amount,
    Status,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum PaymentMethods {
    Table,
    Id,
}
