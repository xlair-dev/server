mod error;
mod mutation;
mod query;

use domain::{
    entity::user::User,
    repository::user::{UserRepository, UserRepositoryError},
};
use sea_orm::{ActiveModelTrait, DbConn};
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

use crate::entities;
use error::convert_user_insert_error;
use mutation::{increment_credits as mutate_increment_credits, save_user};
use query::{find_by_card as query_by_card, find_by_id as query_by_id};

#[cfg(test)]
mod tests;

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
        let card_id = user.card().to_owned();
        let db_user: entities::users::ActiveModel = user.into();

        let db_user_model = db_user.insert(self.db.as_ref()).await.map_err(|err| {
            error!(error = %err, "Failed to insert user");
            convert_user_insert_error(err, &card_id)
        })?;

        info!(user_id = %db_user_model.id, "User persisted by repository");
        Ok(db_user_model.into())
    }

    #[instrument(skip(self), fields(card = %card))]
    async fn find_by_card(&self, card: &str) -> Result<Option<User>, UserRepositoryError> {
        query_by_card(self.db.as_ref(), card).await
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn increment_credits(&self, user_id: &str) -> Result<u32, UserRepositoryError> {
        mutate_increment_credits(self.db.as_ref(), user_id).await
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
