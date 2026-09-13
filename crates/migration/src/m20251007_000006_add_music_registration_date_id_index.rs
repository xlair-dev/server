use sea_orm_migration::prelude::*;
use sea_query::Index;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_musics_registration_date_id")
                    .table(Musics::Table)
                    .col(Musics::RegistrationDate)
                    .col(Musics::Id)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_musics_registration_date_id")
                    .table(Musics::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Musics {
    Table,
    RegistrationDate,
    Id,
}
