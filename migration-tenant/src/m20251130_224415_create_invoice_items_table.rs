use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InvoiceItems::Table)
                    .if_not_exists()
                    .col(pk_uuid(InvoiceItems::Id))
                    .col(
                        uuid_uniq(InvoiceItems::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(uuid(InvoiceItems::InvoiceId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-invoice_items-invoice_id")
                            .from(InvoiceItems::Table, InvoiceItems::InvoiceId)
                            .to(BillingInvoices::Table, BillingInvoices::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string_null(InvoiceItems::ItemType))
                    .col(uuid_null(InvoiceItems::ItemId))
                    .col(text_null(InvoiceItems::Description))
                    .col(decimal(InvoiceItems::Amount))
                    .col(timestamp_null(InvoiceItems::DeletedAt))
                    .col(
                        timestamp(InvoiceItems::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(InvoiceItems::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InvoiceItems::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum InvoiceItems {
    Table,
    Id,
    Pid,
    InvoiceId,
    ItemType,
    ItemId,
    Description,
    Amount,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BillingInvoices {
    Table,
    Id,
}
