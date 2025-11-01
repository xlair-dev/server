use std::sync::Arc;

use domain::repository::{Repositories, record::RecordRepositoryError, user::UserRepositoryError};
use thiserror::Error;

pub mod credits;
pub mod options;
pub mod records;
pub mod register;
pub mod search;
pub mod update;

#[derive(Debug, Error)]
pub enum UserUsecaseError {
    #[error(transparent)]
    UserRepositoryError(#[from] UserRepositoryError),
    #[error("User not found for card: {card}")]
    NotFoundByCard { card: String },
    #[error("User not found for id: {user_id}")]
    NotFoundById { user_id: String },
    #[error(transparent)]
    RecordRepositoryError(RecordRepositoryError),
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
