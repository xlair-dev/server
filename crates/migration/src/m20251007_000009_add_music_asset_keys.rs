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
                    .rename_column(Musics::Jacket, Musics::JacketKey)
                    .add_column(ColumnDef::new(Musics::MusicKey).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .exec_stmt(
                Query::update()
                    .table(Musics::Table)
                    .value(Musics::JacketKey, Expr::cust("NULL"))
                    .and_where(Expr::col(Musics::JacketKey).eq(""))
                    .to_owned(),
            )
            .await?;
        manager
            .exec_stmt(
                Query::update()
                    .table(Musics::Table)
                    .value(
                        Musics::JacketKey,
                        Expr::cust("regexp_replace(jacket_key, '^https?://[^/]+/', '')"),
                    )
                    .and_where(Expr::col(Musics::JacketKey).like("http%"))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Sheets::Table)
                    .add_column(ColumnDef::new(Sheets::ChartKey).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sheets::Table)
                    .drop_column(Sheets::ChartKey)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Musics::Table)
                    .drop_column(Musics::MusicKey)
                    .rename_column(Musics::JacketKey, Musics::Jacket)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Musics {
    Table,
    Jacket,
    JacketKey,
    MusicKey,
}

#[derive(DeriveIden)]
enum Sheets {
    Table,
    ChartKey,
}
