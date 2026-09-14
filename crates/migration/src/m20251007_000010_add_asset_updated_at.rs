use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Musics::Table)
                    .rename_column(Musics::MusicKey, Musics::AudioKey)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Musics::Table)
                    .add_column(
                        ColumnDef::new(Musics::JacketUpdatedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Musics::Table)
                    .add_column(
                        ColumnDef::new(Musics::AudioUpdatedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE "musics"
                SET "jacket_updated_at" = CURRENT_TIMESTAMP
                WHERE "jacket_key" IS NOT NULL;
                "#,
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE "musics"
                SET "audio_updated_at" = CURRENT_TIMESTAMP
                WHERE "audio_key" IS NOT NULL;
                "#,
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE "musics"
                ADD CONSTRAINT "ck_musics_jacket_asset_consistency"
                    CHECK (("jacket_key" IS NULL) = ("jacket_updated_at" IS NULL));
                "#,
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE "musics"
                ADD CONSTRAINT "ck_musics_audio_asset_consistency"
                    CHECK (("audio_key" IS NULL) = ("audio_updated_at" IS NULL));
                "#,
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Sheets::Table)
                    .add_column(
                        ColumnDef::new(Sheets::ChartUpdatedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE "sheets"
                SET "chart_updated_at" = CURRENT_TIMESTAMP
                WHERE "chart_key" IS NOT NULL;
                "#,
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE "sheets"
                ADD CONSTRAINT "ck_sheets_chart_asset_consistency"
                    CHECK (("chart_key" IS NULL) = ("chart_updated_at" IS NULL));
                "#,
            )
            .await
            .map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE "sheets"
                DROP CONSTRAINT IF EXISTS "ck_sheets_chart_asset_consistency";
                "#,
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Sheets::Table)
                    .drop_column(Sheets::ChartUpdatedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE "musics"
                DROP CONSTRAINT IF EXISTS "ck_musics_jacket_asset_consistency";
                "#,
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE "musics"
                DROP CONSTRAINT IF EXISTS "ck_musics_audio_asset_consistency";
                "#,
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Musics::Table)
                    .drop_column(Musics::AudioUpdatedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Musics::Table)
                    .drop_column(Musics::JacketUpdatedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Musics::Table)
                    .rename_column(Musics::AudioKey, Musics::MusicKey)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Musics {
    Table,
    JacketUpdatedAt,
    AudioKey,
    AudioUpdatedAt,
    MusicKey,
}

#[derive(DeriveIden)]
enum Sheets {
    Table,
    ChartUpdatedAt,
}
