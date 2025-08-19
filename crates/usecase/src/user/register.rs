use domain::entity::user::User;

use crate::user::{
    dto::{UserDataDto, UserRegisterDto},
    UserUsecase, UserUsecaseError,
};

impl UserUsecase {
    pub fn register(&self, raw_user: UserRegisterDto) -> Result<UserDataDto, UserUsecaseError> {
        let user = User::new_temporary(raw_user.card, raw_user.display_name);
        let user = self.repositories.user().create(user)?;
        Ok(user.into())
    }
}
