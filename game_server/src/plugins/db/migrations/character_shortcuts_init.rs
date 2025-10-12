use super::characters_init::Characters;
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum CharacterShortcuts {
    Table,
    CharId,
    SlotId,
    ClassVariant,
    Kind,
    ShortcutId,
    Level,
}

#[derive(DeriveMigrationName)]
pub struct CharacterShortcutsMigration;

#[async_trait::async_trait]
impl MigrationTrait for CharacterShortcutsMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CharacterShortcuts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CharacterShortcuts::CharId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CharacterShortcuts::SlotId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CharacterShortcuts::ClassVariant)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CharacterShortcuts::Kind)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CharacterShortcuts::ShortcutId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CharacterShortcuts::Level).integer().null())
                    .primary_key(
                        Index::create()
                            .col(CharacterShortcuts::CharId)
                            .col(CharacterShortcuts::SlotId)
                            .col(CharacterShortcuts::ClassVariant),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_char_id")
                            .from_tbl(CharacterShortcuts::Table)
                            .from_col(CharacterShortcuts::CharId)
                            .to_tbl(Characters::Table)
                            .to_col(Characters::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CharacterShortcuts::Table).to_owned())
            .await
    }
}
