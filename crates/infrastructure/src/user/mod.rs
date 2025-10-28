mod adapter;
mod read;
mod write;

use std::sync::Arc;

use domain::{
    entity::user::User,
    repository::user::{UserRepository, UserRepositoryError},
};
use read::{find_by_card as query_by_card, find_by_id as query_by_id};
use sea_orm::DbConn;
use tracing::{debug, info, instrument};
use write::{create_user, increment_credits as mutate_increment_credits, save_user};

pub struct UserRepositoryImpl {
    db: Arc<DbConn>,
}

impl UserRepositoryImpl {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

impl UserRepository for UserRepositoryImpl {
    #[instrument(skip(self, user), fields(card = %user.card()))]
    async fn create(&self, user: User) -> Result<User, UserRepositoryError> {
        debug!("Persisting user via SeaORM");
        let created = create_user(self.db.as_ref(), user).await?;
        info!(user_id = %created.id(), "User persisted by repository");
        Ok(created)
    }

    #[instrument(skip(self), fields(card = %card))]
    async fn find_by_card(&self, card: &str) -> Result<Option<User>, UserRepositoryError> {
        query_by_card(self.db.as_ref(), card).await
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn increment_credits(&self, user_id: &str) -> Result<u32, UserRepositoryError> {
        mutate_increment_credits(self.db.as_ref(), user_id).await?;
        let user = query_by_id(self.db.as_ref(), user_id)
            .await?
            .ok_or_else(|| UserRepositoryError::NotFound(user_id.to_owned()))?;

        let credits = *user.credits();
        info!(user_id = %user.id(), credits, "User credits incremented successfully");
        Ok(credits)
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_by_id(&self, user_id: &str) -> Result<Option<User>, UserRepositoryError> {
        query_by_id(self.db.as_ref(), user_id).await
    }

    #[instrument(skip(self, user), fields(user_id = %user.id()))]
    async fn save(&self, user: User) -> Result<User, UserRepositoryError> {
        save_user(self.db.as_ref(), user).await
    }
}
