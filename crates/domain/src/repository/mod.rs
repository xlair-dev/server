use crate::repository::user::{MockUserRepository, UserRepository};

pub mod user;

pub trait Repositories {
    type UserRepositoryImpl: UserRepository;

    fn user(&self) -> &Self::UserRepositoryImpl;
}

pub struct MockRepositories {
    pub user: MockUserRepository,
}

impl Repositories for MockRepositories {
    type UserRepositoryImpl = MockUserRepository;

    fn user(&self) -> &Self::UserRepositoryImpl {
        &self.user
    }
}
