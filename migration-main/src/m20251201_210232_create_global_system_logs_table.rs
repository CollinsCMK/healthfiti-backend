use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GlobalSystemLogs::Table)
                    .if_not_exists()
                    .col(pk_auto(GlobalSystemLogs::Id))
                    .col(
                        uuid_uniq(GlobalSystemLogs::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(string(GlobalSystemLogs::EventType))
                    .col(text(GlobalSystemLogs::Message))
                    .col(uuid_uniq(GlobalSystemLogs::SsoUserId).null())
                    .col(integer_null(GlobalSystemLogs::TenantId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-system_logs-tenant_id")
                            .from(GlobalSystemLogs::Table, GlobalSystemLogs::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(string(GlobalSystemLogs::Source))
                    .col(string(GlobalSystemLogs::IpAddress))
                    .col(integer(GlobalSystemLogs::StatusCode))
                    .col(string(GlobalSystemLogs::RequestUrl))
                    .col(string_null(GlobalSystemLogs::Method))
                    .col(timestamp_null(GlobalSystemLogs::DeletedAt))
                    .col(
                        timestamp(GlobalSystemLogs::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(GlobalSystemLogs::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        let _idx_logs_sso_user_id = Index::create()
            .name("idx_logs_sso_user_id")
            .table(GlobalSystemLogs::Table)
            .col(GlobalSystemLogs::SsoUserId)
            .to_owned();

        let _idx_logs_tenant_id = Index::create()
            .name("idx_logs_tenant_id")
            .table(GlobalSystemLogs::Table)
            .col(GlobalSystemLogs::TenantId)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GlobalSystemLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GlobalSystemLogs {
    Table,
    Id,
    Pid,
    EventType,
    Message,
    SsoUserId,
    TenantId,
    Source,
    IpAddress,
    StatusCode,
    RequestUrl,
    Method,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
}
