use sea_orm_migration::prelude::*;
use sea_query::{Expr, Index, extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(ClearType::Table)
                    .values([
                        ClearType::Failed,
                        ClearType::Clear,
                        ClearType::FullCombo,
                        ClearType::AllPerfect,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Records::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Records::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Records::UserId).uuid().not_null())
                    .col(ColumnDef::new(Records::SheetId).uuid().not_null())
                    .col(ColumnDef::new(Records::Score).integer().not_null())
                    .col(
                        ColumnDef::new(Records::ClearType)
                            .custom(ClearType::Table)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Records::PlayCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Records::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_records_user")
                            .from(Records::Table, Records::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_records_sheet")
                            .from(Records::Table, Records::SheetId)
                            .to(Sheets::Table, Sheets::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uk_records_user_sheet")
                    .table(Records::Table)
                    .col(Records::UserId)
                    .col(Records::SheetId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            CREATE TRIGGER trg_records_set_updated_at
            BEFORE UPDATE ON "records"
            FOR EACH ROW
            EXECUTE FUNCTION set_updated_at_timestamp();
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            DROP TRIGGER IF EXISTS trg_records_set_updated_at ON "records";
            "#,
        )
        .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("uk_records_user_sheet")
                    .table(Records::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Records::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(ClearType::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Records {
    Table,
    Id,
    UserId,
    SheetId,
    Score,
    ClearType,
    PlayCount,
    UpdatedAt,
}

#[derive(DeriveIden)]
#[sea_orm(iden = "clear_type")]
enum ClearType {
    Table,
    Failed,
    Clear,
    FullCombo,
    AllPerfect,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Sheets {
    Table,
    Id,
}
