use tokio::net::TcpListener;

use presentation::{config::Config, env, route::create_app, state::State};

#[tokio::main]
async fn main() {
    dotenvy::dotenv_override().expect("Failed to load .env file");

    let postgres_url = env::postgres_url();
    let repositories = infrastructure::RepositoriesImpl::new_default(&postgres_url).await;

    let config = Config::default();
    let state = State::new(config, repositories);

    let app = create_app(state);

    let addr = format!("{}:{}", env::host(), env::port());
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
