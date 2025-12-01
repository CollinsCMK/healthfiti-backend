use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TenantFeatures::Table)
                    .if_not_exists()
                    .col(pk_auto(TenantFeatures::Id))
                    .col(
                        uuid_uniq(TenantFeatures::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(TenantFeatures::TenantId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tenant_features-tenant_id")
                            .from(TenantFeatures::Table, TenantFeatures::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(TenantFeatures::FeatureId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tenant_features-feature_id")
                            .from(TenantFeatures::Table, TenantFeatures::FeatureId)
                            .to(Features::Table, Features::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(boolean(TenantFeatures::IsEnabled).default(true))
                    .col(
                        timestamp(TenantFeatures::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(TenantFeatures::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(timestamp_null(TenantFeatures::DeletedAt))
                    .to_owned(),
            )
            .await?;

            let _uniq_tenant_feature = Index::create()
                .name("uniq_tenant_feature")
                .table(TenantFeatures::Table)
                .col(TenantFeatures::TenantId)
                .col(TenantFeatures::FeatureId)
                .unique()
                .to_owned();
            
            let _idx_tenant_features_tenant_id = Index::create()
                .name("idx_tenant_features_tenant_id")
                .table(TenantFeatures::Table)
                .col(TenantFeatures::TenantId)
                .to_owned();
            
            let _idx_tenant_features_feature_id = Index::create()
                .name("idx_tenant_features_feature_id")
                .table(TenantFeatures::Table)
                .col(TenantFeatures::FeatureId)
                .to_owned();
            
            let _idx_tenant_features_is_enabled = Index::create()
                .name("idx_tenant_features_is_enabled")
                .table(TenantFeatures::Table)
                .col(TenantFeatures::IsEnabled)
                .to_owned();
            
            let _idx_composite = Index::create()
                .name("idx_tenant_features_tenant_feature_enabled")
                .table(TenantFeatures::Table)
                .col(TenantFeatures::TenantId)
                .col(TenantFeatures::FeatureId)
                .col(TenantFeatures::IsEnabled)
                .to_owned();
            
            Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TenantFeatures::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TenantFeatures {
    Table,
    Id,
    Pid,
    TenantId,
    FeatureId,
    IsEnabled,
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