use super::characters_init::Characters;
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum Items {
    Table,
    ObjectId,
    OwnerId,
    ItemId,
    Count,
    EnchantLevel,
    Location,
    LocationData,
    Mana,
    Time,
    X,
    Y,
    Z,
    ElementsInfo,
}

#[derive(DeriveMigrationName)]
pub struct ItemsMigration;

#[async_trait::async_trait]
impl MigrationTrait for ItemsMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Items::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Items::ObjectId)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Items::OwnerId).integer())
                    .col(ColumnDef::new(Items::ItemId).integer().not_null())
                    .col(ColumnDef::new(Items::Count).big_integer().default(1))
                    .col(
                        ColumnDef::new(Items::EnchantLevel)
                            .small_integer()
                            .default(0),
                    )
                    .col(ColumnDef::new(Items::Location).small_integer().not_null())
                    .col(ColumnDef::new(Items::LocationData).integer())
                    .col(ColumnDef::new(Items::Mana).integer())
                    .col(ColumnDef::new(Items::Time).integer())
                    .col(ColumnDef::new(Items::X).integer())
                    .col(ColumnDef::new(Items::Y).integer())
                    .col(ColumnDef::new(Items::Z).integer())
                    .col(ColumnDef::new(Items::ElementsInfo).json())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_item_owner")
                            .from_tbl(Items::Table)
                            .from_col(Items::OwnerId)
                            .to_tbl(Characters::Table)
                            .to_col(Characters::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_item_owner")
                    .table(Items::Table)
                    .col(Items::OwnerId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_item_owner").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Items::Table).to_owned())
            .await
    }
}
