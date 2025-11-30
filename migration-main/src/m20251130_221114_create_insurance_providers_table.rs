use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InsuranceProviders::Table)
                    .if_not_exists()
                    .col(pk_uuid(InsuranceProviders::Id))
                    .col(
                        uuid_uniq(InsuranceProviders::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(string(InsuranceProviders::Name))
                    .col(string_null(InsuranceProviders::Email))
                    .col(string_null(InsuranceProviders::CountryCode))
                    .col(string_null(InsuranceProviders::PhoneNumber))
                    .col(string_null(InsuranceProviders::Website))
                    .col(timestamp_null(InsuranceProviders::DeletedAt))
                    .col(
                        timestamp(InsuranceProviders::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(InsuranceProviders::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        let _idx_insurance_providers_phone_unique = Index::create()
            .unique()
            .name("idx_insurance_providers_phone_unique")
            .table(InsuranceProviders::Table)
            .col(InsuranceProviders::CountryCode)
            .col(InsuranceProviders::PhoneNumber)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InsuranceProviders::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum InsuranceProviders {
    Table,
    Id,
    Pid,
    Name,
    Email,
    CountryCode,
    PhoneNumber,
    Website,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}
