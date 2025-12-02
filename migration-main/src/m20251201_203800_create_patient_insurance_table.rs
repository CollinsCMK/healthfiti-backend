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
                    .as_enum(Alias::new("policyholder_relationship"))
                    .values([
                        Alias::new("yourself"),
                        Alias::new("spouse"),
                        Alias::new("parent"),
                        Alias::new("child"),
                        Alias::new("other"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("verification_status"))
                    .values([
                        Alias::new("pending"),
                        Alias::new("verified"),
                        Alias::new("invalid"),
                        Alias::new("expired"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PatientInsurance::Table)
                    .if_not_exists()
                    .col(pk_auto(PatientInsurance::Id))
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
                    .col(integer(PatientInsurance::InsuranceId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-patient_insurance-insurance_id")
                            .from(PatientInsurance::Table, PatientInsurance::InsuranceId)
                            .to(InsuranceProviders::Table, InsuranceProviders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string(PatientInsurance::PolicyNumber).string_len(100))
                    .col(string_null(PatientInsurance::GroupNumber).string_len(100))
                    .col(string_null(PatientInsurance::PlanName).string_len(255))
                    .col(string_null(PatientInsurance::PlanType).string_len(100))
                    .col(date_null(PatientInsurance::CoverageStartDate))
                    .col(date_null(PatientInsurance::CoverageEndDate))
                    .col(string_null(PatientInsurance::PolicyHolderName).string_len(255))
                    .col(enumeration_null(
                        PatientInsurance::PolicyHolderRelationship,
                        Alias::new("policyholder_relationship"),
                        vec![
                            Alias::new("yourself"),
                            Alias::new("spouse"),
                            Alias::new("parent"),
                            Alias::new("child"),
                            Alias::new("other"),
                        ],
                    ))
                    .col(decimal_null(PatientInsurance::CopayAmount).decimal_len(10, 2))
                    .col(decimal_null(PatientInsurance::DeductibleAmount).decimal_len(10, 2))
                    .col(
                        decimal(PatientInsurance::DeductibleMetYtd)
                            .decimal_len(10, 2)
                            .default(0.00),
                    )
                    .col(decimal_null(PatientInsurance::OutOfPocketMax).decimal_len(10, 2))
                    .col(
                        decimal(PatientInsurance::OutOfPocketMetYtd)
                            .decimal_len(10, 2)
                            .default(0.00),
                    )
                    .col(
                        enumeration(
                            PatientInsurance::VerificationStatus,
                            Alias::new("verification_status"),
                            vec![
                                Alias::new("pending"),
                                Alias::new("verified"),
                                Alias::new("invalid"),
                                Alias::new("expired"),
                            ],
                        )
                        .default("pending"),
                    )
                    .col(timestamp_null(PatientInsurance::VerifiedAt))
                    .col(boolean(PatientInsurance::IsPrimary).default(true))
                    .col(text_null(PatientInsurance::CardFrontImage))
                    .col(text_null(PatientInsurance::CardBackImage))
                    .col(text_null(PatientInsurance::Notes))
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
            .await?;

        let _idx_patient_id = Index::create()
            .name("idx_insurance_patient_id")
            .table(PatientInsurance::Table)
            .col(PatientInsurance::PatientId)
            .to_owned();

        let _idx_insurance_provider_id = Index::create()
            .name("idx_insurance_provider_id")
            .table(PatientInsurance::Table)
            .col(PatientInsurance::InsuranceId)
            .to_owned();

        let _idx_policy_number = Index::create()
            .name("idx_insurance_policy_number")
            .table(PatientInsurance::Table)
            .col(PatientInsurance::PolicyNumber)
            .to_owned();

        let _idx_verification_status = Index::create()
            .name("idx_insurance_verification_status")
            .table(PatientInsurance::Table)
            .col(PatientInsurance::VerificationStatus)
            .to_owned();

        let _uniq_patient_provider_policy = Index::create()
            .unique()
            .name("uniq_insurance_patient_provider_policy")
            .table(PatientInsurance::Table)
            .col(PatientInsurance::PatientId)
            .col(PatientInsurance::InsuranceId)
            .col(PatientInsurance::PolicyNumber)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PatientInsurance::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("policyholder_relationship"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("verification_status"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum PatientInsurance {
    Table,
    Id,
    Pid,
    PatientId,
    InsuranceId,
    PolicyNumber,
    GroupNumber,
    PlanName,
    PlanType,
    CoverageStartDate,
    CoverageEndDate,
    PolicyHolderName,
    PolicyHolderRelationship,
    CopayAmount,
    DeductibleAmount,
    DeductibleMetYtd,
    OutOfPocketMax,
    OutOfPocketMetYtd,
    VerificationStatus,
    VerifiedAt,
    IsPrimary,
    CardFrontImage,
    CardBackImage,
    Notes,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum InsuranceProviders {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Patients {
    Table,
    Id,
}
