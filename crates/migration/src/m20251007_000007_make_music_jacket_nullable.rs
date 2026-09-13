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
                    .modify_column(ColumnDef::new(Musics::Jacket).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::update()
                    .table(Musics::Table)
                    .value(Musics::Jacket, "")
                    .and_where(Expr::col(Musics::Jacket).is_null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Musics::Table)
                    .modify_column(ColumnDef::new(Musics::Jacket).string().not_null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Musics {
    Table,
    Jacket,
}
