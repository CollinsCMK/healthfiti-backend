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
                    .col(integer(SubscriptionPlanFeatures::SubscriptionPlanId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-subscription_plan_features-subscription_plan_id")
                            .from(
                                SubscriptionPlanFeatures::Table,
                                SubscriptionPlanFeatures::SubscriptionPlanId,
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
                                SubscriptionPlanFeatures::FeatureId,
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
            .await?;
        
        let _idx_uniq_subscription_plan_feature = Index::create()
            .name("uniq_subscription_plan_feature")
            .table(SubscriptionPlanFeatures::Table)
            .col(SubscriptionPlanFeatures::SubscriptionPlanId)
            .col(SubscriptionPlanFeatures::FeatureId)
            .unique()
            .to_owned();

        let _idx_subscription_plan_id = Index::create()
            .name("idx_subscription_plan_features_subscription_plan_id")
            .table(SubscriptionPlanFeatures::Table)
            .col(SubscriptionPlanFeatures::SubscriptionPlanId)
            .to_owned();

        let _idx_feature_id = Index::create()
            .name("idx_subscription_plan_features_feature_id")
            .table(SubscriptionPlanFeatures::Table)
            .col(SubscriptionPlanFeatures::FeatureId)
            .to_owned();

        Ok(())
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
    SubscriptionPlanId,
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