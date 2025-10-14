pub use sea_orm_migration::prelude::*;

mod m20251007_000001_create_users_table;
mod m20251007_000002_create_musics_table;
mod m20251007_000003_create_sheets_table;
mod m20251007_000004_create_records_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251007_000001_create_users_table::Migration),
            Box::new(m20251007_000002_create_musics_table::Migration),
            Box::new(m20251007_000003_create_sheets_table::Migration),
            Box::new(m20251007_000004_create_records_table::Migration),
        ]
    }
}
