use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct AccountsMigration;

#[async_trait::async_trait]
impl MigrationTrait for AccountsMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Accounts::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Accounts::Name).string().not_null())
                    .col(ColumnDef::new(Accounts::Password).string().not_null())
                    .col(ColumnDef::new(Accounts::Email).string().null())
                    .col(ColumnDef::new(Accounts::CreatedTime).timestamp().not_null())
                    .col(
                        ColumnDef::new(Accounts::AccessLevel)
                            .small_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Accounts::LastIp).string().null())
                    .col(
                        ColumnDef::new(Accounts::LastServer)
                            .small_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
    Name,
    Password,
    Email,
    CreatedTime,
    AccessLevel,
    LastIp,
    LastServer,
}
