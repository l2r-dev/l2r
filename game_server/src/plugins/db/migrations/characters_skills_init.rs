use crate::plugins::db::migrations::characters_init::Characters;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct CharacterSkillsMigration;

#[derive(DeriveIden)]
pub enum CharacterSkills {
    Table,
    CharId,
    SkillId,
    SkillLevel,
    SubClass,
}

#[async_trait::async_trait]
impl MigrationTrait for CharacterSkillsMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CharacterSkills::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CharacterSkills::CharId).integer().not_null())
                    .col(
                        ColumnDef::new(CharacterSkills::SkillId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CharacterSkills::SubClass)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(CharacterSkills::SkillLevel)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(CharacterSkills::CharId)
                            .col(CharacterSkills::SkillId)
                            .col(CharacterSkills::SubClass),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_char_id")
                            .from_tbl(CharacterSkills::Table)
                            .from_col(CharacterSkills::CharId)
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
            .drop_table(Table::drop().table(CharacterSkills::Table).to_owned())
            .await
    }
}
