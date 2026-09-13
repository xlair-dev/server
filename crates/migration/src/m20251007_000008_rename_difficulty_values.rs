use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TYPE difficulty RENAME VALUE 'easy' TO 'basic';
                 ALTER TYPE difficulty RENAME VALUE 'normal' TO 'advanced';
                 ALTER TYPE difficulty RENAME VALUE 'hard' TO 'master';",
            )
            .await
            .map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TYPE difficulty RENAME VALUE 'basic' TO 'easy';
                 ALTER TYPE difficulty RENAME VALUE 'advanced' TO 'normal';
                 ALTER TYPE difficulty RENAME VALUE 'master' TO 'hard';",
            )
            .await
            .map(|_| ())
    }
}
