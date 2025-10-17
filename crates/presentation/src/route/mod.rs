use axum::{routing::get, routing::post, Router};
use tower_http::trace::TraceLayer;

use crate::state::State;

pub mod user;

pub fn create_app(state: State) -> Router {
    let users = Router::new().route("/", post(user::handle_post));
    let health = Router::new().route("/", get(|| async { "OK" }));

    // TODO: Add auth middleware
    let private_routes = Router::new().nest("/users", users);

    let public_routes = Router::new().nest("/health", health);

    // TODO: Add cors layer
    Router::new()
        .merge(private_routes)
        .merge(public_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
