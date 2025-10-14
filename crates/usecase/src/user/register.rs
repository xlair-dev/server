use domain::{
    entity::user::User,
    repository::{user::UserRepository, Repositories},
};

use crate::{
    model::user::{UserDataDto, UserRegisterDto},
    user::{UserUsecase, UserUsecaseError},
};

impl<R: Repositories> UserUsecase<R> {
    pub async fn register(
        &self,
        raw_user: UserRegisterDto,
    ) -> Result<UserDataDto, UserUsecaseError> {
        let user = User::new_temporary(raw_user.card, raw_user.display_name);
        let user = self.repositories.user().create(user).await?;
        Ok(user.into())
    }
}
