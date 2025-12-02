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
                    .as_enum(Alias::new("aggregation_period"))
                    .values([
                        Alias::new("hourly"),
                        Alias::new("daily"),
                        Alias::new("monthly"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UsageMetrics::Table)
                    .if_not_exists()
                    .col(pk_auto(UsageMetrics::Id))
                    .col(
                        uuid_uniq(UsageMetrics::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer_null(UsageMetrics::TenantId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-usage_metrics-tenant_id")
                            .from(UsageMetrics::Table, UsageMetrics::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(string(UsageMetrics::MetricType).string_len(100))
                    .col(big_integer(UsageMetrics::MetricValue))
                    .col(enumeration(
                        UsageMetrics::AggregationPeriod,
                        Alias::new("aggregation_period"),
                        vec![
                            Alias::new("hourly"),
                            Alias::new("daily"),
                            Alias::new("monthly"),
                        ],
                    ))
                    .col(timestamp(UsageMetrics::PeriodStart))
                    .col(timestamp(UsageMetrics::PeriodEnd))
                    .col(uuid_null(UsageMetrics::FacilityId))
                    .col(json_null(UsageMetrics::Metadata))
                    .col(timestamp_null(UsageMetrics::DeletedAt))
                    .col(
                        timestamp(UsageMetrics::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(UsageMetrics::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        let _idx_usage_metrics_tenant_id = Index::create()
            .name("idx_usage_metrics_tenant_id")
            .table(UsageMetrics::Table)
            .col(UsageMetrics::TenantId)
            .to_owned();

        let _idx_usage_metrics_metric_type = Index::create()
            .name("idx_usage_metrics_metric_type")
            .table(UsageMetrics::Table)
            .col(UsageMetrics::MetricType)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UsageMetrics::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("aggregation_period"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum UsageMetrics {
    Table,
    Id,
    Pid,
    TenantId,
    MetricType,
    MetricValue,
    AggregationPeriod,
    PeriodStart,
    PeriodEnd,
    FacilityId,
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
