use crate::repository::user::UserRepository;

pub mod user;

pub trait Repositories {
    type UserRepositoryImpl: UserRepository;

    fn user_repository(&self) -> &Self::UserRepositoryImpl;
}
