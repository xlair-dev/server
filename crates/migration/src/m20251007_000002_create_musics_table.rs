use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Musics::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Musics::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Musics::Title).string().not_null())
                    .col(ColumnDef::new(Musics::Artist).string().not_null())
                    .col(ColumnDef::new(Musics::Bpm).decimal_len(6, 3).not_null())
                    .col(ColumnDef::new(Musics::Genre).integer().not_null())
                    .col(ColumnDef::new(Musics::Jacket).string().not_null())
                    .col(
                        ColumnDef::new(Musics::RegistrationDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Musics::IsTest)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Musics::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Musics {
    Table,
    Id,
    Title,
    Artist,
    Bpm,
    Genre,
    Jacket,
    RegistrationDate,
    IsTest,
}
