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
                    .as_enum(Alias::new("billing_cycle"))
                    .values([
                        Alias::new("weekly"),
                        Alias::new("monthly"),
                        Alias::new("quartely"),
                        Alias::new("yearly"),
                        Alias::new("custom"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SubscriptionPlans::Table)
                    .if_not_exists()
                    .col(pk_auto(SubscriptionPlans::Id))
                    .col(
                        uuid_uniq(SubscriptionPlans::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(string(SubscriptionPlans::Name).string_len(100))
                    .col(string_uniq(SubscriptionPlans::Slug).string_len(50))
                    .col(text_null(SubscriptionPlans::Description))
                    .col(decimal_null(SubscriptionPlans::PriceWeekly).decimal_len(10, 2))
                    .col(decimal_null(SubscriptionPlans::PriceMonthly).decimal_len(10, 2))
                    .col(decimal_null(SubscriptionPlans::PriceQuartely).decimal_len(10, 2))
                    .col(decimal_null(SubscriptionPlans::PriceYearly).decimal_len(10, 2))
                    .col(integer(SubscriptionPlans::TrialDays).default(7))
                    .col(integer_null(SubscriptionPlans::MaxFacilities))
                    .col(integer_null(SubscriptionPlans::MaxUsers))
                    .col(integer_null(SubscriptionPlans::MaxPatientsPerMonth))
                    .col(integer_null(SubscriptionPlans::StorageGb))
                    .col(integer_null(SubscriptionPlans::ApiRateLimitPerHour))
                    .col(
                        enumeration(
                            SubscriptionPlans::BillingCycle,
                            Alias::new("billing_cycle"),
                            vec![
                                Alias::new("weekly"),
                                Alias::new("monthly"),
                                Alias::new("quartely"),
                                Alias::new("yearly"),
                                Alias::new("custom"),
                            ],
                        )
                        .default("monthly"),
                    )
                    .col(
                        decimal(SubscriptionPlans::SetupFee)
                            .decimal_len(10, 2)
                            .default(0.00),
                    )
                    .col(boolean(SubscriptionPlans::IsPublic).default(true))
                    .col(boolean(SubscriptionPlans::IsActive).default(true))
                    .col(timestamp_null(SubscriptionPlans::DeletedAt))
                    .col(
                        timestamp(SubscriptionPlans::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(SubscriptionPlans::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        let _idx_subscription_plans_slug = Index::create()
            .name("idx_subscription_plans_slug")
            .table(SubscriptionPlans::Table)
            .col(SubscriptionPlans::Slug)
            .to_owned();

        let _idx_subscription_plans_billing_cycle = Index::create()
            .name("idx_subscription_plans_billing_cycle")
            .table(SubscriptionPlans::Table)
            .col(SubscriptionPlans::BillingCycle)
            .to_owned();

        let _idx_subscription_plans_is_active = Index::create()
            .name("idx_subscription_plans_is_active")
            .table(SubscriptionPlans::Table)
            .col(SubscriptionPlans::IsActive)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SubscriptionPlans::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Alias::new("billing_cycle")).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SubscriptionPlans {
    Table,
    Id,
    Pid,
    Name,
    Slug,
    Description,
    PriceWeekly,
    PriceMonthly,
    PriceQuartely,
    PriceYearly,
    TrialDays,
    MaxFacilities,
    MaxUsers,
    MaxPatientsPerMonth,
    StorageGb,
    ApiRateLimitPerHour,
    BillingCycle,
    SetupFee,
    IsPublic,
    IsActive,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}
