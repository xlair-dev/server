use domain::{
    entity::user::User,
    repository::user::{UserRepository, UserRepositoryError},
};
use sea_orm::{ActiveModelTrait, DbConn};

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
    async fn create(&self, user: User) -> Result<User, UserRepositoryError> {
        let db_user: entities::users::ActiveModel = user.into();

        let user = db_user
            .insert(&self.db)
            .await
            .map_err(anyhow::Error::from)?;

        Ok(user.into())
    }
}
