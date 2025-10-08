use domain::repository::Repositories;

pub mod entities;
pub mod user;

pub struct RepositoriesImpl {
    user: user::UserRepositoryImpl,
}

impl RepositoriesImpl {
    pub fn new(user: user::UserRepositoryImpl) -> Self {
        Self { user }
    }
}

impl Repositories for RepositoriesImpl {
    type UserRepositoryImpl = user::UserRepositoryImpl;

    fn user(&self) -> &Self::UserRepositoryImpl {
        &self.user
    }
}
