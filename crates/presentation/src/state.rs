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
    pub asset_storage: Option<Arc<dyn usecase::asset::AssetStorage>>,
}

impl State {
    pub fn new(config: Config, repositories: RepositoriesImpl) -> Self {
        let repositories = Arc::new(repositories);
        let usecases = Arc::new(usecase::Usecases::new(repositories));
        Self {
            usecases,
            config,
            asset_storage: None,
        }
    }

    pub fn with_asset_storage(
        mut self,
        asset_storage: impl usecase::asset::AssetStorage + 'static,
    ) -> Self {
        self.asset_storage = Some(Arc::new(asset_storage));
        self
    }
}
