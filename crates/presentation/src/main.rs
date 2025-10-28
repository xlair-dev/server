use presentation::{config::Config, env, route::create_app, state::State};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    load_env();
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

/// Loads environment variables from the `.env` file if present. Implicitly depends on environment
/// variables provided by the surrounding process environment when `.env` is absent.
fn load_env() {
    match dotenvy::dotenv_override() {
        Ok(_) => {}
        Err(dotenvy::Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("Failed to load .env file: {error}"),
    }
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
