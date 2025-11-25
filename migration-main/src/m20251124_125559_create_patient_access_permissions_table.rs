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
                    .as_enum(Alias::new("patient_permissions_access_level"))
                    .values([Alias::new("read"), Alias::new("write"), Alias::new("full")])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PatientAccessPermissions::Table)
                    .if_not_exists()
                    .col(pk_auto(PatientAccessPermissions::Id))
                    .col(
                        uuid_uniq(PatientAccessPermissions::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(PatientAccessPermissions::TenantId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-patient_access_permissions-tenant_id")
                            .from(
                                PatientAccessPermissions::Table,
                                PatientAccessPermissions::TenantId,
                            )
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(PatientAccessPermissions::PatientId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-patient_access_permissions-patient_id")
                            .from(
                                PatientAccessPermissions::Table,
                                PatientAccessPermissions::PatientId,
                            )
                            .to(Patients::Table, Patients::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(enumeration(
                        PatientAccessPermissions::AccessLevel,
                        Alias::new("patient_permissions_access_level"),
                        vec![Alias::new("read"), Alias::new("write"), Alias::new("full")],
                    ))
                    .col(timestamp_null(PatientAccessPermissions::ExpiresAt))
                    .col(timestamp_null(PatientAccessPermissions::DeletedAt))
                    .col(
                        timestamp(PatientAccessPermissions::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(PatientAccessPermissions::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        let _idx_tenant_subscription = Index::create()
            .unique()
            .name("idx_tenant_subscription")
            .table(PatientAccessPermissions::Table)
            .col(PatientAccessPermissions::TenantId)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(PatientAccessPermissions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("patient_permissions_access_level"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum PatientAccessPermissions {
    Table,
    Id,
    Pid,
    TenantId,
    PatientId,
    AccessLevel,
    DeletedAt,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Patients {
    Table,
    Id,
}
