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
                    .as_enum(Alias::new("subscription_plans_billing_cycle"))
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
                    .table(SubscriptionPlans::Table)
                    .if_not_exists()
                    .col(pk_auto(SubscriptionPlans::Id))
                    .col(
                        uuid_uniq(SubscriptionPlans::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(string_uniq(SubscriptionPlans::Name))
                    .col(text(SubscriptionPlans::Description))
                    .col(decimal(SubscriptionPlans::Price))
                    .col(enumeration(
                        SubscriptionPlans::BillingCycle,
                        Alias::new("subscription_plans_billing_cycle"),
                        vec![
                            Alias::new("weekly"),
                            Alias::new("monthly"),
                            Alias::new("quarterly"),
                            Alias::new("yearly"),
                        ],
                    ))
                    .col(integer(SubscriptionPlans::MaxPharmacies))
                    .check(Expr::col(SubscriptionPlans::MaxPharmacies).gt(Expr::val(0)))
                    .col(integer(SubscriptionPlans::MaxUsers))
                    .check(Expr::col(SubscriptionPlans::MaxUsers).gt(Expr::val(0)))
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
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SubscriptionPlans::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("subscription_plans_billing_cycle"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SubscriptionPlans {
    Table,
    Id,
    Pid,
    Name,
    Description,
    Price,
    BillingCycle,
    MaxPharmacies,
    MaxUsers,
    IsActive,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}
