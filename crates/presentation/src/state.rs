use std::sync::Arc;

use crate::config::Config;

// TODO: Use real implementations when available
#[cfg(not(test))]
pub type RepositoriesImpl = infrastructure::RepositoriesImpl;

#[cfg(test)]
pub type RepositoriesImpl = domain::repository::MockRepositories;

#[derive(Clone)]
pub struct State {
    pub usecases: Arc<usecase::Usecases<RepositoriesImpl>>,
    pub config: Config,
}

impl State {
    pub fn new(config: Config, repositories: RepositoriesImpl) -> Self {
        let repositories = Arc::new(repositories);
        let usecases = Arc::new(usecase::Usecases::new(repositories));
        Self { usecases, config }
    }
}
