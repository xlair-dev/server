use crate::config::Config;

#[derive(Clone)]
pub struct State {
    pub usecases: usecase::Usecases,
    pub config: Config,
}

impl State {
    pub fn new(usecases: usecase::Usecases, config: Config) -> Self {
        Self { usecases, config }
    }
}
