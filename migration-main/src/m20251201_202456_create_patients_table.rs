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
                    .as_enum(Alias::new("gender"))
                    .values([
                        Alias::new("male"),
                        Alias::new("female"),
                        Alias::new("other"),
                        Alias::new("prefer_not_to_say"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Patients::Table)
                    .if_not_exists()
                    .col(pk_auto(Patients::Id))
                    .col(
                        uuid_uniq(Patients::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(uuid_uniq(Patients::SsoUserId).null())
                    .col(string_null(Patients::FirstName).string_len(100))
                    .col(string_null(Patients::LastName).string_len(100))
                    .col(string_null(Patients::MiddleName).string_len(100))
                    .col(string(Patients::PreferredLanguage).string_len(10).default("en"))
                    .col(string_uniq(Patients::PhotoUrl).null())
                    .col(date_null(Patients::Dob))
                    .col(enumeration_null(
                        Patients::Gender,
                        Alias::new("gender"),
                        vec![
                            Alias::new("male"),
                            Alias::new("female"),
                            Alias::new("other"),
                            Alias::new("prefer_not_to_say"),
                        ],
                    ))
                    .col(string_uniq(Patients::NationalID).string_len(50).null())
                    .col(string_uniq(Patients::PassportNumber).string_len(50).null())
                    .col(string_uniq(Patients::Email).string_len(255).null())
                    .col(string_null(Patients::CountryCode))
                    .col(string_uniq(Patients::PhoneNumber).string_len(20).null())
                    .col(text_null(Patients::Address).string_len(255))
                    .col(string_null(Patients::City).string_len(100))
                    .col(string_null(Patients::County).string_len(100))
                    .col(string_null(Patients::Country).string_len(100))
                    .col(uuid_null(Patients::PrimaryTenantId))
                    .col(string_null(Patients::BloodType).string_len(5))
                    .col(array_null(
                        Patients::Allergies,
                        ColumnType::String(StringLen::None),
                    ))
                    .col(array_null(
                        Patients::MedicalConditions,
                        ColumnType::String(StringLen::None),
                    ))
                    .col(json_binary_null(Patients::EmergencyContact))
                    .col(timestamp_null(Patients::DeletedAt))
                    .col(
                        timestamp(Patients::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(Patients::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        let _idx_patients_email = Index::create()
            .unique()
            .name("idx_patients_email")
            .table(Patients::Table)
            .col(Patients::Email)
            .to_owned();

        let _idx_patients_phone = Index::create()
            .unique()
            .name("idx_patients_phone")
            .table(Patients::Table)
            .col(Patients::CountryCode)
            .col(Patients::PhoneNumber)
            .to_owned();

        let _idx_patients_national_id = Index::create()
            .unique()
            .name("idx_patients_national_id")
            .table(Patients::Table)
            .col(Patients::NationalID)
            .to_owned();

        let _idx_patients_sso_user_id = Index::create()
            .unique()
            .name("idx_patients_sso_user_id")
            .table(Patients::Table)
            .col(Patients::SsoUserId)
            .to_owned();

        let _idx_patients_primary_tenant_id = Index::create()
            .unique()
            .name("idx_patients_primary_tenant_id")
            .table(Patients::Table)
            .col(Patients::PrimaryTenantId)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Patients::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Alias::new("gender")).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Patients {
    Table,
    Id,
    Pid,
    SsoUserId,
    FirstName,
    LastName,
    MiddleName,
    PreferredLanguage,
    PhotoUrl,
    Dob,
    Gender,
    NationalID,
    PassportNumber,
    Email,
    CountryCode,
    PhoneNumber,
    Address,
    City,
    County,
    Country,
    PrimaryTenantId,
    BloodType,
    Allergies,
    MedicalConditions,
    EmergencyContact,
    DeletedAt,
    UpdatedAt,
    CreatedAt,
}