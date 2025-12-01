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
                    .as_enum(Alias::new("billing_item_type"))
                    .values([
                        Alias::new("subscription_fee"),
                        Alias::new("overage"),
                        Alias::new("addon"),
                        Alias::new("support"),
                        Alias::new("custom"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(BillingLineItems::Table)
                    .if_not_exists()
                    .col(pk_auto(BillingLineItems::Id))
                    .col(
                        uuid_uniq(BillingLineItems::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(BillingLineItems::TenantId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-billing_line_items-tenant_id")
                            .from(BillingLineItems::Table, BillingLineItems::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(BillingLineItems::SubscriptionId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-billing_line_items-subscription_id")
                            .from(BillingLineItems::Table, BillingLineItems::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(date(BillingLineItems::BillingPeriodStart))
                    .col(date(BillingLineItems::BillingPeriodEnd))
                    .col(enumeration(
                        BillingLineItems::ItemType,
                        Alias::new("billing_item_type"),
                        vec![
                            Alias::new("subscription_fee"),
                            Alias::new("overage"),
                            Alias::new("addon"),
                            Alias::new("support"),
                            Alias::new("custom"),
                        ],
                    ))
                    .col(text(BillingLineItems::Description))
                    .col(integer(BillingLineItems::Quantity).default(1))
                    .col(decimal(BillingLineItems::UnitPrice).decimal_len(10, 2))
                    .col(decimal(BillingLineItems::TotalAmount).decimal_len(10, 2))
                    .col(json_null(BillingLineItems::Metadata))
                    .col(timestamp_null(BillingLineItems::DeletedAt))
                    .col(
                        timestamp(BillingLineItems::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(BillingLineItems::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        let _idx_billing_line_items_tenant_id = Index::create()
            .name("idx_billing_line_items_tenant_id")
            .table(BillingLineItems::Table)
            .col(BillingLineItems::TenantId)
            .to_owned();

        let _idx_billing_line_items_subscription_id = Index::create()
            .name("idx_billing_line_items_subscription_id")
            .table(BillingLineItems::Table)
            .col(BillingLineItems::SubscriptionId)
            .to_owned();

        let _idx_billing_line_items_billing_period_start = Index::create()
            .name("idx_billing_line_items_billing_period_start")
            .table(BillingLineItems::Table)
            .col(BillingLineItems::BillingPeriodStart)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BillingLineItems::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("billing_item_type"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum BillingLineItems {
    Table,
    Id,
    Pid,
    TenantId,
    SubscriptionId,
    BillingPeriodStart,
    BillingPeriodEnd,
    ItemType,
    Description,
    Quantity,
    UnitPrice,
    TotalAmount,
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
