use std::sync::Arc;

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

#[derive(Clone)]
pub struct UserUsecase {
    repositories: Arc<dyn Repositories>,
}

impl UserUsecase {
    pub fn new(repositories: Arc<dyn Repositories>) -> Self {
        Self { repositories }
    }
}
