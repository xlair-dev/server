use domain::{
    entity::user::User,
    repository::{user::UserRepository, Repositories},
};

use crate::user::{
    dto::{UserDataDto, UserRegisterDto},
    UserUsecase, UserUsecaseError,
};

impl<R: Repositories> UserUsecase<R> {
    pub fn register(&self, raw_user: UserRegisterDto) -> Result<UserDataDto, UserUsecaseError> {
        let user = User::new_temporary(raw_user.card, raw_user.display_name);
        let user = self.repositories.user().create(user)?;
        Ok(user.into())
    }
}
