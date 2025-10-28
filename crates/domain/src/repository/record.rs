use std::future::Future;

use mockall::automock;
use thiserror::Error;

use crate::entity::{level::Level, record::Record};

#[derive(Debug, Error)]
pub enum RecordRepositoryError {
    #[error("User not found: {0}")]
    /// Relies on the user table to enforce referential integrity; returned when the persistence
    /// layer cannot find the associated owner for the requested records.
    UserNotFound(String),
    #[error("Sheet not found: {0}")]
    /// Raised when the persistence layer cannot resolve the referenced sheet while mutating
    /// records. Depends on the `fk_records_sheet` foreign key to surface the violation.
    SheetNotFound(String),
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct RecordWithMetadata {
    pub record: Record,
    pub level: Level,
    pub is_test: bool,
}

impl RecordWithMetadata {
    pub fn new(record: Record, level: Level, is_test: bool) -> Self {
        Self {
            record,
            level,
            is_test,
        }
    }
}

#[automock]
pub trait RecordRepository: Send + Sync {
    fn find_by_user_id(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<Record>, RecordRepositoryError>> + Send;
    fn find_with_metadata_by_user_id(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<RecordWithMetadata>, RecordRepositoryError>> + Send;

    /// Loads records for the specified user and sheet IDs. More efficient than loading all records
    /// when only a subset is needed. Returns an empty vector if none of the specified sheet IDs
    /// have records.
    fn find_by_user_id_and_sheet_ids(
        &self,
        user_id: &str,
        sheet_ids: &[String],
    ) -> impl Future<Output = Result<Vec<Record>, RecordRepositoryError>> + Send;

    /// Persists a new record aggregate. Callers must guarantee that the record identifier is
    /// unique; the repository will generate an error if the tuple `(user_id, sheet_id)` already
    /// exists.
    fn insert(
        &self,
        record: Record,
    ) -> impl Future<Output = Result<Record, RecordRepositoryError>> + Send;

    /// Persists changes to an existing record aggregate, matching rows by the primary key.
    fn update(
        &self,
        record: Record,
    ) -> impl Future<Output = Result<Record, RecordRepositoryError>> + Send;
}
