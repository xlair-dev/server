use std::sync::Arc;

use domain::repository::Repositories;

pub mod model;
pub mod user;

pub struct Usecases<R: Repositories> {
    pub user: user::UserUsecase<R>,
}

impl<R: Repositories> Usecases<R> {
    pub fn new(repositories: Arc<R>) -> Self {
        Self {
            user: user::UserUsecase::new(repositories),
        }
    }
}

impl<R: Repositories> Clone for Usecases<R> {
    fn clone(&self) -> Self {
        Self {
            user: self.user.clone(),
        }
    }
}
