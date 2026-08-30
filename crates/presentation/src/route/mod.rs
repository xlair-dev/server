use axum::{
    Router,
    http::{HeaderValue, Method, header},
    middleware,
    routing::{get, patch, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    auth::{Authenticator, AuthorizationPolicy, PrincipalKind},
    env::allowed_origin,
    state::State,
};

pub mod admin;
pub mod ranking;
pub mod statistics;
pub mod sync;
pub mod user;

pub fn create_app(state: State, authenticator: Option<Authenticator>) -> Router {
    let users = Router::new()
        .route("/", post(user::handle_post))
        .route("/", get(user::handle_get))
        .route("/{userId}", patch(user::handle_update_user))
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
    let sync_route = Router::new().route("/", get(sync::handle_get));
    let statistics_route = Router::new().route("/summary", get(statistics::handle_get_summary));
    let ranking_route = Router::new()
        .route("/sheets/{sheetId}", get(ranking::handle_get_sheet_ranking))
        .route("/total-score", get(ranking::handle_get_total_ranking))
        .route("/rating", get(ranking::handle_get_rating_ranking))
        .route("/xp", get(ranking::handle_get_xp_ranking));
    let health = Router::new().route("/", get(|| async { "OK" }));
    let admin_routes =
        Router::new().route("/db/synchronize", post(admin::handle_db_synchronization));

    let private_routes = Router::new()
        .nest("/users", users)
        .nest("/sync", sync_route);
    let private_routes = if let Some(authenticator) = authenticator.clone() {
        private_routes
            .layer(middleware::from_fn_with_state(
                AuthorizationPolicy::only(PrincipalKind::Device),
                crate::auth::authorize,
            ))
            .layer(middleware::from_fn_with_state(
                authenticator,
                crate::auth::middleware,
            ))
    } else {
        private_routes
    };

    let admin_routes = if let Some(authenticator) = authenticator {
        admin_routes
            .layer(middleware::from_fn_with_state(
                AuthorizationPolicy::only(PrincipalKind::Admin),
                crate::auth::authorize,
            ))
            .layer(middleware::from_fn_with_state(
                authenticator,
                crate::auth::middleware,
            ))
    } else {
        admin_routes
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
        .nest("/admin", admin_routes)
        .merge(public_routes)
        .fallback(crate::route::not_found)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

async fn not_found() -> crate::error::AppError {
    crate::error::AppError::not_found()
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{self, Body},
        http::Request,
    };
    use domain::repository::{
        MockRepositories, music::MockMusicRepository, record::MockRecordRepository,
        user::MockUserRepository,
    };
    use reqwest::Client;
    use serde_json::Value;
    use tower::ServiceExt;

    use super::*;

    fn build_authenticated_router() -> Router {
        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: MockRecordRepository::new(),
            music: MockMusicRepository::new(),
        };
        let state = State::new(crate::config::Config::default(), repositories);
        let authenticator = Authenticator::new(
            Client::new(),
            "https://issuer.example.com".into(),
            "https://api.example.com".into(),
            "dashboard-client-id".into(),
        );
        create_app(state, Some(authenticator))
    }

    #[tokio::test]
    async fn private_route_requires_authentication() {
        let response = build_authenticated_router()
            .oneshot(Request::get("/sync").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn admin_route_requires_authentication() {
        let response = build_authenticated_router()
            .oneshot(
                Request::post("/admin/db/synchronize")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn public_route_does_not_require_authentication() {
        let response = build_authenticated_router()
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_route_returns_json_not_found_body() {
        let response = build_authenticated_router()
            .oneshot(Request::get("/unknown").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["error"], "Resource not found");
    }

    #[tokio::test]
    async fn unsupported_method_returns_method_not_allowed() {
        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: MockRecordRepository::new(),
            music: MockMusicRepository::new(),
        };
        let state = State::new(crate::config::Config::default(), repositories);
        let response = create_app(state, None)
            .oneshot(
                Request::get("/users/00000000-0000-0000-0000-000000000000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            axum::http::StatusCode::METHOD_NOT_ALLOWED
        );
    }
}
