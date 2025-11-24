use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Features::Table)
                    .if_not_exists()
                    .col(pk_auto(Features::Id))
                    .col(
                        uuid_uniq(Features::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(string_uniq(Features::Name))
                    .col(text_null(Features::Description))
                    .col(
                        timestamp(Features::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(Features::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(timestamp_null(Features::DeletedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Features::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Features {
    Table,
    Id,
    Pid,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}