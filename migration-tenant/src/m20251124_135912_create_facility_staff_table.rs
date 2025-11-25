use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FacilityStaff::Table)
                    .if_not_exists()
                    .col(pk_auto(FacilityStaff::Id))
                    .col(
                        uuid_uniq(FacilityStaff::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(FacilityStaff::FacilityId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-facility_staff-facility_id")
                            .from(FacilityStaff::Table, FacilityStaff::FacilityId)
                            .to(Facilities::Table, Facilities::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(FacilityStaff::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-facility_staff-user_id")
                            .from(FacilityStaff::Table, FacilityStaff::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string_null(FacilityStaff::Position))
                    .col(boolean(FacilityStaff::IsManager).default(false))
                    .col(timestamp_null(FacilityStaff::DeletedAt))
                    .col(
                        timestamp(FacilityStaff::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(FacilityStaff::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FacilityStaff::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum FacilityStaff {
    Table,
    Id,
    Pid,
    FacilityId,
    UserId,
    Position,
    IsManager,
    DeletedAt,
    UpdatedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Facilities {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
