use std::sync::Arc;

use domain::repository::{
    Repositories,
    record::{RecordRepository, RecordRepositoryError},
    user::{UserRepository, UserRepositoryError},
};
use thiserror::Error;

use crate::model::statistics::GlobalStatisticsDto;

#[derive(Debug, Error)]
pub enum StatisticsUsecaseError {
    #[error(transparent)]
    UserRepository(#[from] UserRepositoryError),
    #[error(transparent)]
    RecordRepository(#[from] RecordRepositoryError),
}

pub struct StatisticsUsecase<R: Repositories> {
    repositories: Arc<R>,
}

impl<R: Repositories> StatisticsUsecase<R> {
    pub fn new(repositories: Arc<R>) -> Self {
        Self { repositories }
    }

    pub async fn summary(&self) -> Result<GlobalStatisticsDto, StatisticsUsecaseError> {
        let total_users = self.repositories.user().count_all().await?;
        let total_credits = self.repositories.user().sum_credits().await?;
        let total_score = self.repositories.record().sum_scores().await?;

        Ok(GlobalStatisticsDto::new(
            total_credits,
            total_users,
            total_score,
        ))
    }
}

impl<R: Repositories> Clone for StatisticsUsecase<R> {
    fn clone(&self) -> Self {
        Self {
            repositories: Arc::clone(&self.repositories),
        }
    }
}
