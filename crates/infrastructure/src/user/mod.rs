mod adapter;
mod read;
mod write;

use std::sync::Arc;

use domain::{
    entity::{user::User, user_play_option::UserPlayOption},
    repository::user::{UserRepository, UserRepositoryError},
};
use read::{
    count_all as query_count_users, find_by_card as query_by_card, find_by_id as query_by_id,
    find_play_option as query_play_option, public_users_by_rating as query_public_by_rating,
    public_users_by_xp as query_public_by_xp, sum_credits as query_sum_credits,
};
use sea_orm::DbConn;
use tracing::{debug, info, instrument};
use write::{
    create_user, increment_credits as mutate_increment_credits,
    save_play_option as mutate_play_option, save_user,
};

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

    #[instrument(skip(self))]
    async fn count_all(&self) -> Result<u64, UserRepositoryError> {
        query_count_users(self.db.as_ref()).await
    }

    #[instrument(skip(self))]
    async fn sum_credits(&self) -> Result<u64, UserRepositoryError> {
        query_sum_credits(self.db.as_ref()).await
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_play_option(
        &self,
        user_id: &str,
    ) -> Result<Option<UserPlayOption>, UserRepositoryError> {
        query_play_option(self.db.as_ref(), user_id).await
    }

    #[instrument(skip(self, option), fields(user_id = %option.user_id()))]
    async fn save_play_option(
        &self,
        option: UserPlayOption,
    ) -> Result<UserPlayOption, UserRepositoryError> {
        mutate_play_option(self.db.as_ref(), option).await
    }

    #[instrument(skip(self), fields(limit))]
    async fn find_public_top_by_rating(
        &self,
        limit: u64,
    ) -> Result<Vec<User>, UserRepositoryError> {
        debug!("Fetching public users by rating via SeaORM");
        let users = query_public_by_rating(self.db.as_ref(), limit).await?;
        info!(
            count = users.len(),
            "Public users by rating fetched successfully"
        );
        Ok(users)
    }

    #[instrument(skip(self), fields(limit))]
    async fn find_public_top_by_xp(&self, limit: u64) -> Result<Vec<User>, UserRepositoryError> {
        debug!("Fetching public users by XP via SeaORM");
        let users = query_public_by_xp(self.db.as_ref(), limit).await?;
        info!(
            count = users.len(),
            "Public users by XP fetched successfully"
        );
        Ok(users)
    }
}
