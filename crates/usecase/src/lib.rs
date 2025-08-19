use std::sync::Arc;

pub mod user;

#[derive(Clone)]
pub struct Usecases {
    pub user: user::UserUsecase,
}

impl Usecases {
    pub fn new(repositories: Arc<dyn domain::repository::Repositories>) -> Self {
        Self {
            user: user::UserUsecase::new(repositories),
        }
    }
}
