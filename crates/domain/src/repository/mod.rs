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
        Self {
            user: Box::new(MockUserRepository::default()),
        }
    }
}
