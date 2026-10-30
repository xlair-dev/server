use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserPlayOptions::Table)
                    .drop_column(UserPlayOptions::JudgmentOffset)
                    .add_column(
                        ColumnDef::new(UserPlayOptions::AudioJudgmentTiming)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .add_column(
                        ColumnDef::new(UserPlayOptions::VisualJudgmentTiming)
                            .double()
                            .not_null()
                            .default(0.0),
                    )
                    .add_column(
                        ColumnDef::new(UserPlayOptions::Mirror)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(
                        ColumnDef::new(UserPlayOptions::NoteVolume)
                            .integer()
                            .not_null()
                            .default(100),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserPlayOptions::Table)
                    .add_column(
                        ColumnDef::new(UserPlayOptions::JudgmentOffset)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .drop_column(UserPlayOptions::AudioJudgmentTiming)
                    .drop_column(UserPlayOptions::VisualJudgmentTiming)
                    .drop_column(UserPlayOptions::Mirror)
                    .drop_column(UserPlayOptions::NoteVolume)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum UserPlayOptions {
    Table,
    JudgmentOffset,
    AudioJudgmentTiming,
    VisualJudgmentTiming,
    Mirror,
    NoteVolume,
}
