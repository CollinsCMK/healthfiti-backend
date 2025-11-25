use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServiceReviews::Table)
                    .if_not_exists()
                    .col(pk_auto(ServiceReviews::Id))
                    .col(
                        uuid_uniq(ServiceReviews::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(ServiceReviews::ServiceId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-service_reviews-service_id")
                            .from(ServiceReviews::Table, ServiceReviews::ServiceId)
                            .to(Services::Table, Services::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(uuid(ServiceReviews::SsoUserId))
                    .col(
                        integer(ServiceReviews::Rating).check(
                            Expr::col(ServiceReviews::Rating)
                                .gte(1)
                                .and(Expr::col(ServiceReviews::Rating).lte(5)),
                        ),
                    )
                    .col(text_null(ServiceReviews::Comment))
                    .col(timestamp_null(ServiceReviews::DeletedAt))
                    .col(
                        timestamp(ServiceReviews::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(ServiceReviews::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ServiceReviews::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ServiceReviews {
    Table,
    Id,
    Pid,
    ServiceId,
    SsoUserId,
    Rating,
    Comment,
    DeletedAt,
    UpdatedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Services {
    Table,
    Id,
}
