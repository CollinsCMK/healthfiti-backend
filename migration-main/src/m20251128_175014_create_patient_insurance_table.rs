use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
                    .col(string(PatientInsurance::Provider))
                    .col(string(PatientInsurance::PolicyNumber))
                    .col(string_null(PatientInsurance::GroupNumber))
                    .col(string_null(PatientInsurance::PlanType))
                    .col(date_null(PatientInsurance::CoverageStartDate))
                    .col(date_null(PatientInsurance::CoverageEndDate))
                    .col(boolean(PatientInsurance::IsPrimary).default(true))
                    .col(string_null(PatientInsurance::InsuranceCardFront))
                    .col(string_null(PatientInsurance::InsuranceCardBack))
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

        let _idx_patient_insurance = Index::create()
            .unique()
            .name("idx_patient_insurance_provide_policy")
            .table(PatientInsurance::Table)
            .col(PatientInsurance::PatientId)
            .col(PatientInsurance::Provider)
            .col(PatientInsurance::PolicyNumber)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PatientInsurance::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PatientInsurance {
    Table,
    Id,
    Pid,
    PatientId,
    Provider,
    PolicyNumber,
    GroupNumber,
    PlanType,
    CoverageStartDate,
    CoverageEndDate,
    IsPrimary,
    InsuranceCardFront,
    InsuranceCardBack,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Patients {
    Table,
    Id,
}
