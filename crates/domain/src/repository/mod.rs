use crate::repository::user::UserRepository;

pub mod user;

pub trait Repositories: Send + Sync {
    fn user(&self) -> Box<dyn UserRepository>;
}
