use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct CharactersMigration;

#[derive(DeriveIden)]
pub enum Characters {
    Table,
    Id,
    AccountId,
    IsLastActive,
    Name,
    Title,
    Exp,
    Sp,
    CreatedTime,
    Race,
    SubClass,
    ClassId,
    Appearance,
    Vitals,
    X,
    Y,
    Z,
}

#[async_trait::async_trait]
impl MigrationTrait for CharactersMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Characters::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Characters::Id)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Characters::AccountId).uuid().not_null())
                    .col(
                        ColumnDef::new(Characters::IsLastActive)
                            .boolean()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Characters::Name).string().not_null())
                    .col(ColumnDef::new(Characters::Title).string().not_null())
                    .col(ColumnDef::new(Characters::Exp).big_integer().default(0))
                    .col(ColumnDef::new(Characters::Sp).integer().default(0))
                    .col(
                        ColumnDef::new(Characters::CreatedTime)
                            .timestamp()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_owned()),
                    )
                    .col(ColumnDef::new(Characters::Race).integer().not_null())
                    .col(
                        ColumnDef::new(Characters::SubClass)
                            .small_integer()
                            .default(0),
                    )
                    .col(ColumnDef::new(Characters::ClassId).integer().not_null())
                    .col(ColumnDef::new(Characters::Appearance).json().not_null())
                    .col(ColumnDef::new(Characters::Vitals).json().not_null())
                    .col(ColumnDef::new(Characters::X).integer().not_null())
                    .col(ColumnDef::new(Characters::Y).integer().not_null())
                    .col(ColumnDef::new(Characters::Z).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Characters::Table).to_owned())
            .await
    }
}
