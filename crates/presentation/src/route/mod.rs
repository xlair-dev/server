use axum::{
    Router,
    http::{HeaderValue, Method, header},
    middleware,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{auth::Authenticator, env::allowed_origin, state::State};

pub mod ranking;
pub mod statistics;
pub mod sync;
pub mod user;

pub fn create_app(state: State, authenticator: Option<Authenticator>) -> Router {
    let device_users = Router::new()
        .route("/", post(user::handle_post))
        .route("/", get(user::handle_get))
        .route(
            "/{userId}/records",
            get(user::handle_get_records).post(user::handle_post_records),
        )
        .route(
            "/{userId}/options",
            get(user::handle_get_play_option).post(user::handle_post_play_option),
        )
        .route(
            "/{userId}/credits/increment",
            post(user::handle_increment_credits),
        );
    let user_update = Router::new().route("/{userId}", post(user::handle_update_user));
    let sync_route = Router::new().route("/", get(sync::handle_get));
    let statistics_route = Router::new().route("/summary", get(statistics::handle_get_summary));
    let ranking_route = Router::new()
        .route("/sheets/{sheetId}", get(ranking::handle_get_sheet_ranking))
        .route("/total-score", get(ranking::handle_get_total_ranking))
        .route("/rating", get(ranking::handle_get_rating_ranking))
        .route("/xp", get(ranking::handle_get_xp_ranking));
    let health = Router::new().route("/", get(|| async { "OK" }));

    let device_routes = Router::new()
        .nest("/users", device_users)
        .nest("/sync", sync_route);
    let private_routes = if let Some(authenticator) = authenticator {
        let user_update = user_update.layer(middleware::from_fn_with_state(
            authenticator.clone(),
            crate::auth::middleware,
        ));
        let device_routes = device_routes
            .layer(middleware::from_fn(crate::auth::require_device))
            .layer(middleware::from_fn_with_state(
                authenticator,
                crate::auth::middleware,
            ));
        Router::new()
            .merge(device_routes)
            .nest("/users", user_update)
    } else {
        Router::new()
            .merge(device_routes)
            .nest("/users", user_update)
    };

    let public_routes = Router::new()
        .nest("/health", health)
        .nest("/rankings", ranking_route)
        .nest("/statistics", statistics_route);

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
