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
                    .as_enum(Alias::new("subscription_tier"))
                    .values([
                        Alias::new("basic"),
                        Alias::new("professional"),
                        Alias::new("enterprise"),
                        Alias::new("custom"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("tenant_subscription_status"))
                    .values([
                        Alias::new("trial"),
                        Alias::new("active"),
                        Alias::new("suspended"),
                        Alias::new("cancelled"),
                        Alias::new("expired"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Tenants::Table)
                    .if_not_exists()
                    .col(pk_auto(Tenants::Id))
                    .col(
                        uuid_uniq(Tenants::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(uuid_uniq(Tenants::SsoTenantId))
                    .col(string(Tenants::Name).string_len(255))
                    .col(string_uniq(Tenants::Slug).string_len(100))
                    .col(text_uniq(Tenants::DbUrl))
                    .col(enumeration_null(
                        Tenants::SubscriptionTier,
                        Alias::new("subscription_tier"),
                        vec![
                            Alias::new("basic"),
                            Alias::new("professional"),
                            Alias::new("enterprise"),
                            Alias::new("custom"),
                        ],
                    ))
                    .col(
                        enumeration(
                            Tenants::SubscriptionStatus,
                            Alias::new("tenant_subscription_status"),
                            vec![
                                Alias::new("trial"),
                                Alias::new("active"),
                                Alias::new("suspended"),
                                Alias::new("cancelled"),
                                Alias::new("expired"),
                            ],
                        )
                        .default("trial"),
                    )
                    .col(timestamp_null(Tenants::TrialEndsAt))
                    .col(timestamp_null(Tenants::SubscriptionStartedAt))
                    .col(timestamp_null(Tenants::SubscriptionEndsAt))
                    .col(string_null(Tenants::ContactEmail).string_len(255))
                    .col(string_null(Tenants::ContactPhone).string_len(20))
                    .col(text_null(Tenants::LogoUrl))
                    .col(string(Tenants::Timezone).string_len(50).default("UTC"))
                    .col(string(Tenants::Currency).string_len(3).default("KES"))
                    .col(json_null(Tenants::Settings))
                    .col(boolean(Tenants::OnboardingCompleted).default(false))
                    .col(timestamp_null(Tenants::DeletedAt))
                    .col(
                        timestamp(Tenants::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(Tenants::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        let _idx_tenants_slug = Index::create()
            .name("idx_tenants_slug")
            .table(Tenants::Table)
            .col(Tenants::Slug)
            .to_owned();

        let _idx_tenants_subscription_status = Index::create()
            .name("idx_tenants_subscription_status")
            .table(Tenants::Table)
            .col(Tenants::SubscriptionStatus)
            .to_owned();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tenants::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("subscription_tier"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("tenant_subscription_status"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
    Pid,
    SsoTenantId,
    Name,
    Slug,
    DbUrl,
    SubscriptionTier,
    SubscriptionStatus,
    TrialEndsAt,
    SubscriptionStartedAt,
    SubscriptionEndsAt,
    ContactEmail,
    ContactPhone,
    LogoUrl,
    Timezone,
    Currency,
    Settings,
    OnboardingCompleted,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}
