use sea_orm_migration::prelude::*;

// run migration with: cd crates && DATABASE_URL="<DATABASE_URL>" sea-orm-cli migrate refresh
#[async_std::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
