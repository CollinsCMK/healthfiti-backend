use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tenants::Table)
                    .if_not_exists()
                    .col(pk_auto(Tenants::Id))
                    .col(
                        uuid_uniq(Tenants::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(uuid_uniq(Tenants::SsoTenantId))
                    .col(string(Tenants::DbUrl))
                    .col(integer(Tenants::PlanId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tenants-plan_id")
                            .from(Tenants::Table, Tenants::PlanId)
                            .to(SubscriptionPlans::Table, SubscriptionPlans::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(timestamp_null(Tenants::DeletedAt))
                    .col(
                        timestamp(Tenants::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(Tenants::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
            
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tenants::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
    Pid,
    SsoTenantId,
    DbUrl,
    PlanId,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SubscriptionPlans {
    Table,
    Id,
}
