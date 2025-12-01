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
                    .as_enum(Alias::new("insurance_claim_status"))
                    .values([
                        Alias::new("pending"),
                        Alias::new("approved"),
                        Alias::new("rejected"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(InsuranceClaims::Table)
                    .if_not_exists()
                    .col(pk_uuid(InsuranceClaims::Id))
                    .col(
                        uuid_uniq(InsuranceClaims::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(uuid(InsuranceClaims::MainPatientId))
                    .col(uuid(InsuranceClaims::MainInsuranceId))
                    .col(uuid_null(InsuranceClaims::VisitId))
                    .col(decimal(InsuranceClaims::ClaimAmount))
                    .col(decimal_null(InsuranceClaims::ApprovedAmount))
                    .col(
                        enumeration(
                            InsuranceClaims::Status,
                            Alias::new("insurance_claim_status"),
                            vec![
                                Alias::new("pending"),
                                Alias::new("approved"),
                                Alias::new("rejected"),
                            ],
                        )
                        .default("pending"),
                    )
                    .col(
                        timestamp(InsuranceClaims::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(uuid(InsuranceClaims::SsoTenantId))
                    .col(uuid(InsuranceClaims::MainTenantId))
                    .col(timestamp_null(InsuranceClaims::DeletedAt))
                    .col(
                        timestamp(InsuranceClaims::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InsuranceClaims::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("insurance_claim_status"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum InsuranceClaims {
    Table,
    Id,
    Pid,
    MainPatientId,
    MainInsuranceId,
    VisitId,
    ClaimAmount,
    ApprovedAmount,
    Status,
    SsoTenantId,
    MainTenantId,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}
