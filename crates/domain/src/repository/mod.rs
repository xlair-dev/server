use crate::repository::user::UserRepository;

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
}
