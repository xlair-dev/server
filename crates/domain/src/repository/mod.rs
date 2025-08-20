use crate::repository::user::{MockUserRepository, UserRepository};

pub mod user;

pub struct Repositories {
    pub user: Box<dyn UserRepository>,
}

impl Repositories {
    pub fn new(user_repository: Box<dyn UserRepository>) -> Self {
        Self {
            user: user_repository,
        }
    }

    pub fn new_mock() -> Self {
        // TODO: 以下のような mock は、テスト用に分離する
        let mut mock = MockUserRepository::default();
        mock.expect_create().returning(Ok);
        Self {
            user: Box::new(mock),
        }
    }
}
