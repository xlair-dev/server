use std::future::Future;

use mockall::automock;
use thiserror::Error;

use crate::entity::user::User;

#[derive(Debug, Error)]
pub enum UserRepositoryError {
    #[error("Card ID already exists: {0}")]
    CardIdAlreadyExists(String),
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
}

#[automock]
pub trait UserRepository: Send + Sync {
    fn create(&self, user: User) -> impl Future<Output = Result<User, UserRepositoryError>> + Send;
    fn find_by_card(
        &self,
        card: &str,
    ) -> impl Future<Output = Result<Option<User>, UserRepositoryError>> + Send;
}
