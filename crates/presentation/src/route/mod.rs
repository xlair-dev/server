use axum::{
    Router,
    http::{HeaderValue, Method, header},
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{env::allowed_origin, state::State};

pub mod statistics;
pub mod sync;
pub mod user;

pub fn create_app(state: State) -> Router {
    let users = Router::new()
        .route("/", post(user::handle_post))
        .route("/", get(user::handle_get))
        .route("/{userId}", post(user::handle_update_user))
        .route(
            "/{userId}/records",
            get(user::handle_get_records).post(user::handle_post_records),
        )
        .route(
            "/{userId}/credits/increment",
            post(user::handle_increment_credits),
        );
    let sync_route = Router::new().route("/", get(sync::handle_get));
    let statistics_route = Router::new().route("/summary", get(statistics::handle_get_summary));
    let health = Router::new().route("/", get(|| async { "OK" }));

    // TODO: Add auth middleware
    let private_routes = Router::new()
        .nest("/users", users)
        .nest("/sync", sync_route)
        .nest("/statistics", statistics_route);

    let public_routes = Router::new().nest("/health", health);

    let cors = CorsLayer::new()
        .allow_origin(allowed_origin().parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    Router::new()
        .merge(private_routes)
        .merge(public_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
