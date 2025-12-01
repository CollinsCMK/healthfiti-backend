use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FeatureFlags::Table)
                    .if_not_exists()
                    .col(pk_auto(FeatureFlags::Id))
                    .col(
                        uuid_uniq(FeatureFlags::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(string_uniq(FeatureFlags::FlagName).string_len(100))
                    .col(text_null(FeatureFlags::Description))
                    .col(boolean(FeatureFlags::IsEnabledGlobally).default(false))
                    .col(integer(FeatureFlags::RolloutPercentage).default(0))
                    .col(json_binary_null(FeatureFlags::TargetTiers))
                    .col(json_binary_null(FeatureFlags::TargetTenants))
                    .col(timestamp_null(FeatureFlags::ExpiresAt))
                    .col(
                        timestamp(FeatureFlags::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(FeatureFlags::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(timestamp_null(FeatureFlags::DeletedAt))
                    .to_owned(),
            )
            .await?;

        let _idx_feature_flags_name = Index::create()
            .name("idx_feature_flags_name")
            .table(FeatureFlags::Table)
            .col(FeatureFlags::FlagName)
            .to_owned();

        let _idx_feature_flags_global = Index::create()
            .name("idx_feature_flags_global")
            .table(FeatureFlags::Table)
            .col(FeatureFlags::IsEnabledGlobally)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FeatureFlags::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum FeatureFlags {
    Table,
    Id,
    Pid,
    FlagName,
    Description,
    IsEnabledGlobally,
    RolloutPercentage,
    TargetTiers,
    TargetTenants,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
