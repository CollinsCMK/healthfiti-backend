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
                    .as_enum(Alias::new("subscription_status"))
                    .values([
                        Alias::new("trial"),
                        Alias::new("active"),
                        Alias::new("pending_payment"),
                        Alias::new("past_due"),
                        Alias::new("cancelled"),
                        Alias::new("expired"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Subscriptions::Table)
                    .if_not_exists()
                    .col(pk_auto(Subscriptions::Id))
                    .col(
                        uuid_uniq(Subscriptions::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(Subscriptions::TenantId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-subscriptions-tenant_id")
                            .from(Subscriptions::Table, Subscriptions::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(Subscriptions::PlanId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-subscriptions-plan_id")
                            .from(Subscriptions::Table, Subscriptions::PlanId)
                            .to(SubscriptionPlans::Table, SubscriptionPlans::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        enumeration(
                            Subscriptions::Status,
                            Alias::new("subscription_status"),
                            vec![
                                Alias::new("trial"),
                                Alias::new("active"),
                                Alias::new("pending_payment"),
                                Alias::new("past_due"),
                                Alias::new("cancelled"),
                                Alias::new("expired"),
                            ],
                        )
                        .default("trial"),
                    )
                    .col(timestamp_null(Subscriptions::CurrentPeriodStart))
                    .col(timestamp_null(Subscriptions::CurrentPeriodEnd))
                    .col(boolean(Subscriptions::CancelAtPeriodEnd).default(false))
                    .col(timestamp_null(Subscriptions::CancelledAt))
                    .col(text_null(Subscriptions::CancellationReason))
                    .col(decimal_null(Subscriptions::CustomPrice).decimal_len(10, 2))
                    .col(integer_null(Subscriptions::TrialDays))
                    .col(integer_null(Subscriptions::MaxFacilities))
                    .col(integer_null(Subscriptions::MaxUsers))
                    .col(integer_null(Subscriptions::MaxPatientsPerMonth))
                    .col(integer_null(Subscriptions::StorageGb))
                    .col(integer_null(Subscriptions::ApiRateLimitPerHour))
                    .col(timestamp_null(Subscriptions::DeletedAt))
                    .col(
                        timestamp(Subscriptions::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(Subscriptions::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        let _idx_subscriptions_tenant_id = Index::create()
            .name("idx_subscriptions_tenant_id")
            .table(Subscriptions::Table)
            .col(Subscriptions::TenantId)
            .to_owned();

        let _idx_subscriptions_status = Index::create()
            .name("idx_subscriptions_status")
            .table(Subscriptions::Table)
            .col(Subscriptions::Status)
            .to_owned();

        let _idx_subscriptions_current_period_end = Index::create()
            .name("idx_subscriptions_current_period_end")
            .table(Subscriptions::Table)
            .col(Subscriptions::CurrentPeriodEnd)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("subscription_status"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Subscriptions {
    Table,
    Id,
    Pid,
    TenantId,
    PlanId,
    Status,
    CurrentPeriodStart,
    CurrentPeriodEnd,
    CancelAtPeriodEnd,
    CancelledAt,
    CancellationReason,
    CustomPrice,
    TrialDays,
    MaxFacilities,
    MaxUsers,
    MaxPatientsPerMonth,
    StorageGb,
    ApiRateLimitPerHour,
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
