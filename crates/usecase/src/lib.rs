use std::sync::Arc;

use domain::repository::Repositories;

pub mod user;

#[derive(Clone)]
pub struct Usecases {
    pub user: user::UserUsecase,
}

impl Usecases {
    pub fn new(repositories: Arc<Repositories>) -> Self {
        Self {
            user: user::UserUsecase::new(repositories),
        }
    }
}
