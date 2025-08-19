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

pub struct UserUsecase {
    repositories: Box<dyn Repositories>,
}

impl UserUsecase {
    pub fn new(repositories: Box<dyn Repositories>) -> Self {
        Self { repositories }
    }
}
