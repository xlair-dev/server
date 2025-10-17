use domain::{
    entity::user::User,
    repository::{user::UserRepository, Repositories},
};
use tracing::{debug, info, instrument};

use crate::{
    model::user::{UserDataDto, UserRegisterDto},
    user::{UserUsecase, UserUsecaseError},
};

impl<R: Repositories> UserUsecase<R> {
    #[instrument(skip(self, raw_user), fields(card = %raw_user.card))]
    pub async fn register(
        &self,
        raw_user: UserRegisterDto,
    ) -> Result<UserDataDto, UserUsecaseError> {
        let card = raw_user.card.clone();
        let display_name = raw_user.display_name.clone();
        debug!(%card, %display_name, "Building user aggregate");

        let user = User::new_temporary(raw_user.card, raw_user.display_name);
        let user = self.repositories.user().create(user).await?;
        info!(user_id = %user.id(), "User persisted by repository");
        Ok(user.into())
    }
}
