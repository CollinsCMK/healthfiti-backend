use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SubscriptionPlanFeatures::Table)
                    .if_not_exists()
                    .col(pk_auto(SubscriptionPlanFeatures::Id))
                    .col(
                        uuid_uniq(SubscriptionPlanFeatures::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(SubscriptionPlanFeatures::PlanId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-subscription_plan_features-plan_id")
                            .from(
                                SubscriptionPlanFeatures::Table,
                                SubscriptionPlanFeatures::PlanId,
                            )
                            .to(SubscriptionPlans::Table, SubscriptionPlans::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(SubscriptionPlanFeatures::FeatureId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-subscription_plan_features-feature_id")
                            .from(
                                SubscriptionPlanFeatures::Table,
                                SubscriptionPlanFeatures::PlanId,
                            )
                            .to(Features::Table, Features::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(boolean(SubscriptionPlanFeatures::IsEnabled).default(true))
                    .col(
                        timestamp(SubscriptionPlanFeatures::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(SubscriptionPlanFeatures::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(timestamp_null(SubscriptionPlanFeatures::DeletedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SubscriptionPlanFeatures::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SubscriptionPlanFeatures {
    Table,
    Id,
    Pid,
    PlanId,
    FeatureId,
    IsEnabled,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
enum SubscriptionPlans {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Features {
    Table,
    Id,
}