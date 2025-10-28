use domain::repository::{Repositories, user::UserRepository};
use tracing::{debug, instrument};

use crate::{
    model::user::{UserDataDto, UserUpdateDto},
    user::{UserUsecase, UserUsecaseError},
};

impl<R: Repositories> UserUsecase<R> {
    #[instrument(skip(self, update), fields(user_id = %user_id))]
    pub async fn update_user(
        &self,
        user_id: String,
        update: UserUpdateDto,
    ) -> Result<UserDataDto, UserUsecaseError> {
        debug!("Loading user aggregate for update");
        let mut user = self
            .repositories
            .user()
            .find_by_id(&user_id)
            .await?
            .ok_or(UserUsecaseError::NotFoundById { user_id })?;

        user.set_display_name(update.display_name);
        user.set_is_public(update.is_public);

        let saved = self.repositories.user().save(user).await?;
        Ok(saved.into())
    }
}
