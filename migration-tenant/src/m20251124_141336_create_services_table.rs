use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Services::Table)
                    .if_not_exists()
                    .col(pk_auto(Services::Id))
                    .col(
                        uuid_uniq(Services::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(string_uniq(Services::Name))
                    .col(text_null(Services::Description))
                    .col(integer_null(Services::Price))
                    .col(integer_null(Services::Duration))
                    .col(decimal_len(Services::Rating, 3, 2).default(0.00))
                    .col(integer(Services::TotalReviews).default(0))
                    .col(string_null(Services::Icon))
                    .col(array_null(
                        Services::Features,
                        ColumnType::String(StringLen::None),
                    ))
                    .col(json_binary_null(Services::Metadata))
                    .col(
                        timestamp(Services::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(Services::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(timestamp_null(Services::DeletedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Services::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Services {
    Table,
    Id,
    Pid,
    Name,
    Description,
    Price,
    Duration,
    Rating,
    TotalReviews,
    Icon,
    Features,
    Metadata,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
