use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServiceCategories::Table)
                    .if_not_exists()
                    .col(pk_auto(ServiceCategories::Id))
                    .col(
                        uuid_uniq(ServiceCategories::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(string_uniq(ServiceCategories::Name))
                    .col(text_null(ServiceCategories::Description))
                    .col(json_binary_null(ServiceCategories::Metadata))
                    .col(
                        timestamp(ServiceCategories::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(ServiceCategories::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(timestamp_null(ServiceCategories::DeletedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ServiceCategories::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ServiceCategories {
    Table,
    Id,
    Pid,
    Name,
    Description,
    Metadata,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
