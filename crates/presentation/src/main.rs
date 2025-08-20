use std::sync::Arc;
use tokio::net::TcpListener;

use domain::repository::Repositories;
use presentation::{config::Config, env, route::create_app, state::State};
use usecase::Usecases;

#[tokio::main]
async fn main() {
    let repositories = Repositories::new_mock();
    let usecases = Usecases::new(Arc::new(repositories));
    let config = Config::default();
    let state = State::new(usecases, config);

    let app = create_app(state);

    let addr = format!("{}:{}", env::host(), env::port());
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
