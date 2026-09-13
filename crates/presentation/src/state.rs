use std::sync::Arc;

use crate::config::Config;

#[cfg(not(test))]
pub type RepositoriesImpl = infrastructure::RepositoriesImpl;

#[cfg(test)]
pub type RepositoriesImpl = domain::repository::MockRepositories;

#[derive(Clone)]
pub struct State {
    pub usecases: Arc<usecase::Usecases<RepositoriesImpl>>,
    pub config: Config,
    pub jacket_storage: Option<Arc<dyn usecase::jacket::JacketStorage>>,
}

impl State {
    pub fn new(config: Config, repositories: RepositoriesImpl) -> Self {
        let repositories = Arc::new(repositories);
        let usecases = Arc::new(usecase::Usecases::new(repositories));
        Self {
            usecases,
            config,
            jacket_storage: None,
        }
    }

    pub fn with_jacket_storage(
        mut self,
        jacket_storage: impl usecase::jacket::JacketStorage + 'static,
    ) -> Self {
        self.jacket_storage = Some(Arc::new(jacket_storage));
        self
    }
}
