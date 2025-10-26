use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use presentation::{config::Config, env, route::create_app, state::State};

#[tokio::main]
async fn main() {
    dotenvy::dotenv_override().expect("Failed to load .env file");
    init_tracing();

    let postgres_url = env::postgres_url();
    let repositories = infrastructure::RepositoriesImpl::new_default(&postgres_url).await;

    let config = Config::default();
    let state = State::new(config, repositories);

    let app = create_app(state);

    let addr = format!("{}:{}", env::host(), env::app_port());
    let listener = TcpListener::bind(&addr).await.unwrap();
    info!(%addr, "Starting HTTP server");
    axum::serve(listener, app).await.unwrap();
}

/// Initializes tracing. Implicitly depends on the `RUST_LOG` environment variable to override the filter configuration when present.
fn init_tracing() {
    if tracing::dispatcher::has_been_set() {
        return;
    }

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("presentation=info,tower_http=info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .try_init()
        .ok();
}
