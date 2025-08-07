use domain::repository::{user::UserRepositoryError, Repositories};
use thiserror::Error;

pub mod dto;
pub mod register;

#[derive(Debug, Error)]
pub enum UserUsecaseError {
    #[error(transparent)]
    UserRepositoryError(#[from] UserRepositoryError),
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
}

pub struct UserUsecase<R: Repositories> {
    repositories: R,
}

impl<R: Repositories> UserUsecase<R> {
    pub fn new(repositories: R) -> Self {
        Self { repositories }
    }
}
