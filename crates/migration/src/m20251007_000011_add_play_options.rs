use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Renames the input-judgment offset used against XLAIR's audio playback clock.
/// The existing integer values are milliseconds, as interpreted by the game client.
/// The multi-statement migration relies on PostgreSQL executing a single query atomically.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "ALTER TABLE user_play_options RENAME COLUMN judgment_offset TO audio_judgment_timing;
             ALTER TABLE user_play_options
                ADD COLUMN visual_judgment_timing DOUBLE PRECISION NOT NULL DEFAULT 0,
                ADD COLUMN mirror BOOLEAN NOT NULL DEFAULT FALSE,
                ADD COLUMN note_volume INTEGER NOT NULL DEFAULT 100;"
        ).await.map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE user_play_options
                DROP COLUMN visual_judgment_timing,
                DROP COLUMN mirror,
                DROP COLUMN note_volume;
             ALTER TABLE user_play_options RENAME COLUMN audio_judgment_timing TO judgment_offset;",
            )
            .await
            .map(|_| ())
    }
}
