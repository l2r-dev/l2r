// Re-export migrations from the library
pub use l2r_gameserver::plugins::db::migrations::GameServerMigrator;
use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(GameServerMigrator).await;
}
