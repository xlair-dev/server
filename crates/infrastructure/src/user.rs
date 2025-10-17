use domain::{
    entity::user::User,
    repository::user::{UserRepository, UserRepositoryError},
};
use sea_orm::{ActiveModelTrait, DbConn};
use tracing::{debug, error, info, instrument};

use crate::entities;

pub struct UserRepositoryImpl {
    db: DbConn,
}

impl UserRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

impl UserRepository for UserRepositoryImpl {
    #[instrument(skip(self, user), fields(card = %user.card()))]
    async fn create(&self, user: User) -> Result<User, UserRepositoryError> {
        debug!("Persisting user via SeaORM");
        let db_user: entities::users::ActiveModel = user.into();

        let db_user_model = db_user.insert(&self.db).await.map_err(|err| {
            error!(error = %err, "Failed to insert user");
            anyhow::Error::from(err)
        })?;

        info!(user_id = %db_user_model.id, "User persisted by repository");
        Ok(db_user_model.into())
    }
}
