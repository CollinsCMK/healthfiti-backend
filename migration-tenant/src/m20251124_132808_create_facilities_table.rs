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
            .create_type(
                Type::create()
                    .as_enum(Alias::new("facilities_status"))
                    .values([
                        Alias::new("active"),
                        Alias::new("inactive"),
                        Alias::new("pending_verification"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Facilities::Table)
                    .if_not_exists()
                    .col(pk_auto(Facilities::Id))
                    .col(
                        uuid_uniq(Facilities::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(string(Facilities::Name))
                    .col(text_null(Facilities::Description))
                    .col(string_null(Facilities::FacilityType))
                    .col(array_null(
                        Facilities::Images,
                        ColumnType::String(StringLen::None),
                    ))
                    .col(string_uniq(Facilities::LicenseNumber))
                    .col(string_null(Facilities::CountryCode))
                    .col(string_null(Facilities::PhoneNumber))
                    .col(string_null(Facilities::Email))
                    .col(string_null(Facilities::Website))
                    .col(string(Facilities::Address))
                    .col(string(Facilities::City))
                    .col(string(Facilities::County))
                    .col(string(Facilities::Country))
                    .col(string(Facilities::PostalCode))
                    .col(decimal_len(Facilities::Latitude, 10, 8))
                    .col(decimal_len(Facilities::Longitude, 11, 8))
                    .col(json_binary_null(Facilities::OperatingHours))
                    .col(decimal_len(Facilities::Rating, 3, 2).default(0.00))
                    .col(integer(Facilities::TotalReviews).default(0))
                    .col(
                        enumeration(
                            Facilities::Status,
                            Alias::new("facilities_status"),
                            vec![
                                Alias::new("active"),
                                Alias::new("inactive"),
                                Alias::new("pending_verification"),
                            ],
                        )
                        .default("pending_verification"),
                    )
                    .col(json_binary_null(Facilities::Metadata))
                    .col(boolean(Facilities::IsMainBranch).default(false))
                    .col(timestamp_null(Facilities::DeletedAt))
                    .col(
                        timestamp(Facilities::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(Facilities::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Facilities::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("facilities_status"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Facilities {
    Table,
    Id,
    Pid,
    Name,
    Description,
    FacilityType,
    Images,
    LicenseNumber,
    CountryCode,
    PhoneNumber,
    Email,
    Website,
    Address,
    City,
    County,
    Country,
    PostalCode,
    Latitude,
    Longitude,
    OperatingHours,
    Rating,
    TotalReviews,
    Status,
    Metadata,
    IsMainBranch,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}
