use std::future::Future;

use mockall::automock;
use thiserror::Error;

use crate::entity::user::User;

#[derive(Debug, Error)]
pub enum UserRepositoryError {
    #[error("Card ID already exists: {0}")]
    CardIdAlreadyExists(String),
    #[error("User not found: {0}")]
    /// Database rows are keyed by UUID strings; this error surfaces when the repository cannot
    /// resolve the aggregate in storage while mutating it by its identifier.
    NotFound(String),
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
    /// Executes an atomic credit increment for the aggregate and returns the persisted value.
    /// Implementations must delegate the increment to the storage backend to avoid lost updates
    /// when multiple cabinets consume credits concurrently.
    fn increment_credits(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<u32, UserRepositoryError>> + Send;

    fn find_by_id(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Option<User>, UserRepositoryError>> + Send;

    fn save(&self, user: User) -> impl Future<Output = Result<User, UserRepositoryError>> + Send;

    /// Returns the total number of persisted user aggregates.
    fn count_all(&self) -> impl Future<Output = Result<u64, UserRepositoryError>> + Send;

    /// Sums the `credits` field across all user aggregates. Implementations must default to zero
    /// when the table is empty to keep the operation idempotent for reporting workloads.
    fn sum_credits(&self) -> impl Future<Output = Result<u64, UserRepositoryError>> + Send;
}
