use domain::{
    repository::{Repositories, record::RecordRepository, user::UserRepository},
    service::rating,
};
use tracing::{debug, info, instrument};

use super::{UserUsecase, UserUsecaseError};

impl<R: Repositories> UserUsecase<R> {
    /// Synchronizes persisted user values with the current domain rules.
    #[instrument(skip(self))]
    pub async fn synchronize_db(&self) -> Result<super::DbSynchronizationResult, UserUsecaseError> {
        let users = self.repositories.user().find_all().await?;
        let mut result = super::DbSynchronizationResult::default();

        for mut user in users {
            let user_id = user.id().to_owned();
            let records = self
                .repositories
                .record()
                .find_with_metadata_by_user_id(&user_id)
                .await
                .map_err(UserUsecaseError::RecordRepositoryError)?;
            let new_rating = rating::calculate_user_rating(&records);

            if user.rating().value() != new_rating.value() {
                debug!(%user_id, "Updating rating during database synchronization");
                user.update_rating(new_rating);
                self.repositories.user().save(user).await?;
                result.updated_users += 1;
                result.updated_ratings += 1;
            }
        }

        info!(
            updated_users = result.updated_users,
            "Database synchronization completed"
        );
        Ok(result)
    }
}
