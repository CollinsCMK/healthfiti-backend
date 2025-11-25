use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FacilityReviews::Table)
                    .if_not_exists()
                    .col(pk_auto(FacilityReviews::Id))
                    .col(
                        uuid_uniq(FacilityReviews::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(FacilityReviews::FacilityId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-facility_reviews-facility_id")
                            .from(FacilityReviews::Table, FacilityReviews::FacilityId)
                            .to(Facilities::Table, Facilities::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(uuid(FacilityReviews::SsoUserId))
                    .col(
                        integer(FacilityReviews::Rating).check(
                            Expr::col(FacilityReviews::Rating)
                                .gte(1)
                                .and(Expr::col(FacilityReviews::Rating).lte(5)),
                        ),
                    )
                    .col(text_null(FacilityReviews::Comment))
                    .col(timestamp_null(FacilityReviews::DeletedAt))
                    .col(
                        timestamp(FacilityReviews::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(FacilityReviews::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FacilityReviews::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum FacilityReviews {
    Table,
    Id,
    Pid,
    FacilityId,
    SsoUserId,
    Rating,
    Comment,
    DeletedAt,
    UpdatedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Facilities {
    Table,
    Id,
}
