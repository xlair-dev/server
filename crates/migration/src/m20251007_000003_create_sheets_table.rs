use sea_orm_migration::prelude::*;
use sea_query::{Index, extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Difficulty::Table)
                    .values([
                        Difficulty::Basic,
                        Difficulty::Advanced,
                        Difficulty::Expert,
                        Difficulty::Master,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Sheets::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Sheets::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Sheets::MusicId).uuid().not_null())
                    .col(
                        ColumnDef::new(Sheets::Difficulty)
                            .custom(Difficulty::Table)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Sheets::Level).integer().not_null())
                    .col(ColumnDef::new(Sheets::NotesDesigner).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sheets_music")
                            .from(Sheets::Table, Sheets::MusicId)
                            .to(Musics::Table, Musics::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uk_sheets_music_difficulty")
                    .table(Sheets::Table)
                    .col(Sheets::MusicId)
                    .col(Sheets::Difficulty)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uk_sheets_music_difficulty")
                    .table(Sheets::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Sheets::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Difficulty::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Sheets {
    Table,
    Id,
    MusicId,
    Difficulty,
    Level,
    NotesDesigner,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "difficulty_type")]
enum Difficulty {
    Table,
    Basic,
    Advanced,
    Expert,
    Master,
}

#[derive(DeriveIden)]
enum Musics {
    Table,
    Id,
}
