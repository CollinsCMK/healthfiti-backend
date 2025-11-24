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
                    .as_enum(Alias::new("tenant_subscriptions_status"))
                    .values([
                        Alias::new("active"),
                        Alias::new("cancelled"),
                        Alias::new("past_due"),
                        Alias::new("unpaid"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TenantSubscriptions::Table)
                    .if_not_exists()
                    .col(pk_auto(TenantSubscriptions::Id))
                    .col(
                        uuid_uniq(TenantSubscriptions::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(TenantSubscriptions::TenantId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tenant_subscriptions-tenant_id")
                            .from(TenantSubscriptions::Table, TenantSubscriptions::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(TenantSubscriptions::PlanId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tenant_subscriptions-plan_id")
                            .from(TenantSubscriptions::Table, TenantSubscriptions::PlanId)
                            .to(SubscriptionPlans::Table, SubscriptionPlans::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        enumeration(
                            TenantSubscriptions::Status,
                            Alias::new("tenant_subscriptions_status"),
                            vec![
                                Alias::new("active"),
                                Alias::new("cancelled"),
                                Alias::new("past_due"),
                                Alias::new("unpaid"),
                            ],
                        )
                        .default("active"),
                    )
                    .col(decimal_null(TenantSubscriptions::CustomPrice))
                    .col(integer_null(TenantSubscriptions::MaxPharmacies))
                    .col(integer_null(TenantSubscriptions::MaxUsers))
                    .col(timestamp(TenantSubscriptions::CurrentPeriodStart))
                    .col(timestamp(TenantSubscriptions::CurrentPeriodEnd))
                    .col(timestamp_null(TenantSubscriptions::DeletedAt))
                    .col(
                        timestamp(TenantSubscriptions::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(TenantSubscriptions::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        let _idx_tenant_subscription = Index::create()
            .unique()
            .name("idx_tenant_subscription")
            .table(TenantSubscriptions::Table)
            .col(TenantSubscriptions::TenantId)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TenantSubscriptions::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("tenant_subscriptions_status"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum TenantSubscriptions {
    Table,
    Id,
    Pid,
    TenantId,
    PlanId,
    Status,
    CustomPrice,
    MaxPharmacies,
    MaxUsers,
    CurrentPeriodStart,
    CurrentPeriodEnd,
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
enum SubscriptionPlans {
    Table,
    Id,
}