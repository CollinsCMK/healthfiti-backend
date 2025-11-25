use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FacilityServices::Table)
                    .if_not_exists()
                    .col(pk_auto(FacilityServices::Id))
                    .col(
                        uuid_uniq(FacilityServices::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(FacilityServices::FacilityId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-facility_services-facility_id")
                            .from(FacilityServices::Table, FacilityServices::FacilityId)
                            .to(Facilites::Table, Facilites::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(FacilityServices::ServiceId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-facility_services-service_id")
                            .from(FacilityServices::Table, FacilityServices::ServiceId)
                            .to(Services::Table, Services::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer_null(FacilityServices::Price))
                    .col(boolean(FacilityServices::IsAvailable).default(true))
                    .col(text_null(FacilityServices::Notes))
                    .col(json_binary_null(FacilityServices::Metadata))
                    .col(timestamp_null(FacilityServices::DeletedAt))
                    .col(
                        timestamp(FacilityServices::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(FacilityServices::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FacilityServices::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum FacilityServices {
    Table,
    Id,
    Pid,
    FacilityId,
    ServiceId,
    Price,
    IsAvailable,
    Notes,
    Metadata,
    DeletedAt,
    UpdatedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Services {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Facilites {
    Table,
    Id,
}
