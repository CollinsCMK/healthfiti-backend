use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    schema::*,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_auto(Users::Id))
                    .col(
                        uuid_uniq(Users::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(uuid_uniq(Users::SsoUserId))
                    .col(string_null(Users::PhotoUrl))
                    .col(date_null(Users::Dob))
                    .col(enumeration_null(
                        Users::Gender,
                        Alias::new("gender"),
                        vec![
                            Alias::new("male"),
                            Alias::new("female"),
                            Alias::new("other"),
                        ],
                    ))
                    .col(string_uniq(Users::LicenseNumber).null())
                    .col(string_null(Users::Specialization))
                    .col(text_null(Users::Address))
                    .col(string_null(Users::City))
                    .col(string_null(Users::County))
                    .col(string_null(Users::Country))
                    .col(string_null(Users::PostalCode))
                    .col(json_binary_null(Users::EmergencyContact))
                    .col(timestamp_null(Users::DeletedAt))
                    .col(
                        timestamp(Users::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(Users::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Alias::new("gender")).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Pid,
    SsoUserId,
    PhotoUrl,
    Dob,
    Gender,
    LicenseNumber,
    Specialization,
    Address,
    City,
    County,
    Country,
    PostalCode,
    EmergencyContact,
    DeletedAt,
    UpdatedAt,
    CreatedAt,
}
