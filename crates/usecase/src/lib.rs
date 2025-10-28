use std::sync::Arc;

use domain::repository::Repositories;

pub mod model;
pub mod music;
pub mod user;

pub struct Usecases<R: Repositories> {
    pub user: user::UserUsecase<R>,
    pub music: music::MusicUsecase<R>,
}

impl<R: Repositories> Usecases<R> {
    pub fn new(repositories: Arc<R>) -> Self {
        let music = music::MusicUsecase::new(Arc::clone(&repositories));
        let user = user::UserUsecase::new(repositories);
        Self { user, music }
    }
}

impl<R: Repositories> Clone for Usecases<R> {
    fn clone(&self) -> Self {
        Self {
            user: self.user.clone(),
            music: self.music.clone(),
        }
    }
}
