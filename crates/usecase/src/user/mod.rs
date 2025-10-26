use std::sync::Arc;

use domain::repository::{Repositories, user::UserRepositoryError};
use thiserror::Error;

pub mod register;
pub mod search;

#[derive(Debug, Error)]
pub enum UserUsecaseError {
    #[error(transparent)]
    UserRepositoryError(#[from] UserRepositoryError),
    #[error("User not found for card: {0}")]
    NotFound(String),
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
}

pub struct UserUsecase<R: Repositories> {
    repositories: Arc<R>,
}

impl<R: Repositories> UserUsecase<R> {
    pub fn new(repositories: Arc<R>) -> Self {
        Self { repositories }
    }
}

impl<R: Repositories> Clone for UserUsecase<R> {
    fn clone(&self) -> Self {
        Self {
            repositories: Arc::clone(&self.repositories),
        }
    }
}
