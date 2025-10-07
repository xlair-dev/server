use tokio::net::TcpListener;

use domain::repository::MockRepositories;
use presentation::{config::Config, env, route::create_app, state::State};

#[tokio::main]
async fn main() {
    let repositories = MockRepositories {
        user: domain::repository::user::MockUserRepository::new(),
    };
    let config = Config::default();
    let state = State::new(config, repositories);

    let app = create_app(state);

    let addr = format!("{}:{}", env::host(), env::port());
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
