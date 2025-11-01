use std::sync::Arc;

use domain::repository::Repositories;

pub mod model;
pub mod music;
pub mod statistics;
pub mod user;

pub struct Usecases<R: Repositories> {
    pub user: user::UserUsecase<R>,
    pub music: music::MusicUsecase<R>,
    pub statistics: statistics::StatisticsUsecase<R>,
}

impl<R: Repositories> Usecases<R> {
    pub fn new(repositories: Arc<R>) -> Self {
        let music = music::MusicUsecase::new(Arc::clone(&repositories));
        let user = user::UserUsecase::new(Arc::clone(&repositories));
        let statistics = statistics::StatisticsUsecase::new(repositories);
        Self {
            user,
            music,
            statistics,
        }
    }
}

impl<R: Repositories> Clone for Usecases<R> {
    fn clone(&self) -> Self {
        Self {
            user: self.user.clone(),
            music: self.music.clone(),
            statistics: self.statistics.clone(),
        }
    }
}
