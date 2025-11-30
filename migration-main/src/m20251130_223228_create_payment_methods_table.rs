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
                    .as_enum(Alias::new("payment_method_type"))
                    .values([
                        Alias::new("mpesa"),
                        Alias::new("airtel"),
                        Alias::new("card"),
                        Alias::new("paypal"),
                        Alias::new("cash"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PaymentMethods::Table)
                    .if_not_exists()
                    .col(pk_uuid(PaymentMethods::Id))
                    .col(
                        uuid_uniq(PaymentMethods::Pid)
                            .default(SimpleExpr::Custom("gen_random_uuid()".into())),
                    )
                    .col(integer(PaymentMethods::PatientId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-payment_methods-patient_id")
                            .from(PaymentMethods::Table, PaymentMethods::PatientId)
                            .to(Patients::Table, Patients::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(enumeration(
                        PaymentMethods::Type,
                        Alias::new("payment_method_type"),
                        vec![
                            Alias::new("mpesa"),
                            Alias::new("airtel"),
                            Alias::new("card"),
                            Alias::new("paypal"),
                            Alias::new("cash"),
                        ],
                    ))
                    .col(json_binary(PaymentMethods::Details))
                    .col(boolean(PaymentMethods::IsDefault).default(false))
                    .col(timestamp_null(PaymentMethods::DeletedAt))
                    .col(
                        timestamp(PaymentMethods::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(PaymentMethods::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PaymentMethods::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Alias::new("payment_method_type")).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PaymentMethods {
    Table,
    Id,
    Pid,
    PatientId,
    Type,
    Details,
    IsDefault,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Patients {
    Table,
    Id,
}
