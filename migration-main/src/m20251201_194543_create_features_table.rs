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
                    .col(string_uniq(Features::Code))
                    .col(string(Features::Name))
                    .col(text_null(Features::Description))
                    .col(boolean(Features::IsPremium).default(false))
                    .col(boolean(Features::RequiresSetup).default(false))
                    .col(text_null(Features::SetupInstructions))
                    .col(json_binary_null(Features::Dependencies))
                    .col(boolean(Features::IsActive).default(false))
                    .col(integer(Features::DisplayOrder).default(0))
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
            .await?;

            let _idx_features_code = Index::create()
                .name("idx_features_code")
                .table(Features::Table)
                .col(Features::Code)
                .to_owned();

            let _idx_features_is_premium = Index::create()
                .name("idx_features_is_premium")
                .table(Features::Table)
                .col(Features::IsPremium)
                .to_owned();

            let _idx_features_is_active = Index::create()
                .name("idx_features_is_active")
                .table(Features::Table)
                .col(Features::IsActive)
                .to_owned();

        Ok(())
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
    Code,
    Name,
    Description,
    IsPremium,
    RequiresSetup,
    SetupInstructions,
    Dependencies,
    IsActive,
    DisplayOrder,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}