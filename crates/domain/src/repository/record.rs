use std::future::Future;

use mockall::automock;
use thiserror::Error;

use crate::entity::record::Record;

#[derive(Debug, Error)]
pub enum RecordRepositoryError {
    #[error("User not found: {0}")]
    /// Relies on the user table to enforce referential integrity; returned when the persistence
    /// layer cannot find the associated owner for the requested records.
    UserNotFound(String),
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
}

#[automock]
pub trait RecordRepository: Send + Sync {
    fn find_by_user_id(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<Record>, RecordRepositoryError>> + Send;
}
