// Re-export migrations from the library
pub use l2r_loginserver::plugins::db::migrations::LoginServerMigrator;
use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(LoginServerMigrator).await;
}
