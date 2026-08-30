use domain::{
    repository::{Repositories, record::RecordRepository, user::UserRepository},
    service::rating,
};
use tracing::{debug, info, instrument};

use super::{UserUsecase, UserUsecaseError};

impl<R: Repositories> UserUsecase<R> {
    /// Recalculates persisted ratings for every user from the current rating policy.
    #[instrument(skip(self))]
    pub async fn recalculate_ratings(&self) -> Result<u64, UserUsecaseError> {
        let users = self.repositories.user().find_all().await?;
        let mut updated = 0;

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
                debug!(%user_id, "Updating rating during recalculation");
                user.update_rating(new_rating);
                self.repositories.user().save(user).await?;
                updated += 1;
            }
        }

        info!(updated, "User ratings recalculated");
        Ok(updated)
    }
}
