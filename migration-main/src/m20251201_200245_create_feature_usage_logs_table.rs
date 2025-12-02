use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FeatureUsageLogs::Table)
                    .if_not_exists()
                    .col(pk_auto(FeatureUsageLogs::Id))
                    .col(
                        uuid_uniq(FeatureUsageLogs::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(FeatureUsageLogs::TenantId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-feature_usage_logs-tenant_id")
                            .from(FeatureUsageLogs::Table, FeatureUsageLogs::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(FeatureUsageLogs::FeatureId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-feature_usage_logs-feature_id")
                            .from(FeatureUsageLogs::Table, FeatureUsageLogs::FeatureId)
                            .to(Features::Table, Features::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string(FeatureUsageLogs::UsageType).string_len(50))
                    .col(integer(FeatureUsageLogs::Quantity).default(1))
                    .col(json_null(FeatureUsageLogs::Metadata))
                    .col(
                        timestamp(FeatureUsageLogs::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(FeatureUsageLogs::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(timestamp_null(FeatureUsageLogs::DeletedAt))
                    .to_owned(),
            )
            .await?;

        let _idx_feature_usage_tenant_feature_recorded = Index::create()
            .name("idx_feature_usage_tenant_feature_recorded")
            .table(FeatureUsageLogs::Table)
            .col(FeatureUsageLogs::CreatedAt)
            .to_owned();

        let _idx_feature_usage_feature = Index::create()
            .name("idx_feature_usage_feature")
            .table(FeatureUsageLogs::Table)
            .col(FeatureUsageLogs::FeatureId)
            .to_owned();

        let _idx_feature_usage_tenant = Index::create()
            .name("idx_feature_usage_tenant")
            .table(FeatureUsageLogs::Table)
            .col(FeatureUsageLogs::TenantId)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FeatureUsageLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum FeatureUsageLogs {
    Table,
    Id,
    Pid,
    TenantId,
    FeatureId,
    UsageType,
    Quantity,
    Metadata,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Features {
    Table,
    Id,
}
