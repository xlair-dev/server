use domain::{
    entity::user::User,
    repository::{user::UserRepository, Repositories},
};

use crate::user::{dto::RegisterUserDto, UserUsecase, UserUsecaseError};

impl<R: Repositories> UserUsecase<R> {
    pub fn register(&self, raw_user: RegisterUserDto) -> Result<(), UserUsecaseError> {
        let user = User::new_temporary(raw_user.card, raw_user.display_name);
        self.repositories.user_repository().create(user)?;
        Ok(())
    }
}
