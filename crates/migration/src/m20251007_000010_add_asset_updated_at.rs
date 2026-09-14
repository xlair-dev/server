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
                    .add_column(
                        ColumnDef::new(Musics::JacketUpdatedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(Musics::MusicUpdatedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
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
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sheets::Table)
                    .drop_column(Sheets::ChartUpdatedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Musics::Table)
                    .drop_column(Musics::MusicUpdatedAt)
                    .drop_column(Musics::JacketUpdatedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Musics {
    Table,
    JacketUpdatedAt,
    MusicUpdatedAt,
}

#[derive(DeriveIden)]
enum Sheets {
    Table,
    ChartUpdatedAt,
}
