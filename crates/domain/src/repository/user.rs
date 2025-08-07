use thiserror::Error;

use crate::entity::user::User;

#[derive(Debug, Error)]
pub enum UserRepositoryError {
    #[error("Card ID already exists: {0}")]
    CardIdAlreadyExists(String),
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
}

pub trait UserRepository {
    fn create(&self, user: User) -> Result<User, UserRepositoryError>;
}
