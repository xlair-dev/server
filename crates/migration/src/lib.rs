pub use sea_orm_migration::prelude::*;

mod m20251007_000001_create_users_table;
mod m20251007_000002_create_musics_table;
mod m20251007_000003_create_sheets_table;
mod m20251007_000004_create_records_table;
mod m20251007_000005_create_user_play_options_table;
mod m20251007_000006_add_music_registration_date_id_index;
mod m20251007_000007_make_music_jacket_nullable;
mod m20251007_000008_rename_difficulty_values;
mod m20251007_000009_add_music_asset_keys;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251007_000001_create_users_table::Migration),
            Box::new(m20251007_000002_create_musics_table::Migration),
            Box::new(m20251007_000003_create_sheets_table::Migration),
            Box::new(m20251007_000004_create_records_table::Migration),
            Box::new(m20251007_000005_create_user_play_options_table::Migration),
            Box::new(m20251007_000006_add_music_registration_date_id_index::Migration),
            Box::new(m20251007_000007_make_music_jacket_nullable::Migration),
            Box::new(m20251007_000008_rename_difficulty_values::Migration),
            Box::new(m20251007_000009_add_music_asset_keys::Migration),
        ]
    }
}
