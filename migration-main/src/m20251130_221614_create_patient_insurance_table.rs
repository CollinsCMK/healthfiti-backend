use sea_orm_migration::{prelude::*, schema::*};
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("patient_insurance_status"))
                    .values([
                        Alias::new("active"),
                        Alias::new("inactive"),
                        Alias::new("pending"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PatientInsurance::Table)
                    .if_not_exists()
                    .col(pk_uuid(PatientInsurance::Id))
                    .col(
                        uuid_uniq(PatientInsurance::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(PatientInsurance::PatientId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-patient_insurance-patient_id")
                            .from(PatientInsurance::Table, PatientInsurance::PatientId)
                            .to(Patients::Table, Patients::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(PatientInsurance::ProviderId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-patient_insurance-provider_id")
                            .from(PatientInsurance::Table, PatientInsurance::ProviderId)
                            .to(InsuranceProviders::Table, InsuranceProviders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string(PatientInsurance::PolicyNumber))
                    .col(string_null(PatientInsurance::CoverageType))
                    .col(decimal_null(PatientInsurance::CoverageLimit))
                    .col(date_null(PatientInsurance::EffectiveDate))
                    .col(date_null(PatientInsurance::ExpiryDate))
                    .col(enumeration(
                        PatientInsurance::Status,
                        Alias::new("patient_insurance_status"),
                        vec![
                            Alias::new("active"),
                            Alias::new("inactive"),
                            Alias::new("pending"),
                        ],
                    ).default("active"))
                    .col(boolean(PatientInsurance::Verified).default(false))
                    .col(timestamp_null(PatientInsurance::VerifiedAt))
                    .col(timestamp_null(PatientInsurance::DeletedAt))
                    .col(
                        timestamp(PatientInsurance::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(PatientInsurance::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PatientInsurance::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Alias::new("patient_insurance_status")).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PatientInsurance {
    Table,
    Id,
    Pid,
    PatientId,
    ProviderId,
    PolicyNumber,
    CoverageType,
    CoverageLimit,
    EffectiveDate,
    ExpiryDate,
    Status,
    Verified,
    VerifiedAt,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Patients {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum InsuranceProviders {
    Table,
    Id,
}
