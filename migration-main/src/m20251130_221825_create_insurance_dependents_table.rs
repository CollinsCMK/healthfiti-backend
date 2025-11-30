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
                    .as_enum(Alias::new("insurance_dependent_relationship"))
                    .values([
                        Alias::new("spouse"),
                        Alias::new("child"),
                        Alias::new("parent"),
                        Alias::new("other"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(InsuranceDependents::Table)
                    .if_not_exists()
                    .col(pk_uuid(InsuranceDependents::Id))
                    .col(
                        uuid_uniq(InsuranceDependents::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(InsuranceDependents::InsuranceId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-insurance_dependents-insurance_id")
                            .from(InsuranceDependents::Table, InsuranceDependents::InsuranceId)
                            .to(PatientInsurance::Table, PatientInsurance::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string(InsuranceDependents::Name))
                    .col(enumeration(
                        InsuranceDependents::Relationship,
                        Alias::new("insurance_dependent_relationship"),
                        vec![
                            Alias::new("spouse"),
                            Alias::new("child"),
                            Alias::new("parent"),
                            Alias::new("other"),
                        ],
                    ).null())
                    .col(date_null(InsuranceDependents::DateOfBirth))
                    .col(text_null(InsuranceDependents::CoverageDetails))
                    .col(timestamp_null(InsuranceDependents::DeletedAt))
                    .col(
                        timestamp(InsuranceDependents::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(InsuranceDependents::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InsuranceDependents::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("insurance_dependent_relationship"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum InsuranceDependents {
    Table,
    Id,
    Pid,
    InsuranceId,
    Name,
    Relationship,
    DateOfBirth,
    CoverageDetails,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum PatientInsurance {
    Table,
    Id,
}