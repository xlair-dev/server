pub struct State {
    pub usecases: usecase::Usecases<Box<dyn domain::repository::Repositories>>,
}
