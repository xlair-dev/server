use axum::{Json, extract::State, http::StatusCode};
use tracing::{info, instrument};

use crate::{
    error::AppError,
    model::user::{RegisterUserRequest, UserDataResponse},
};

type AppResult<T> = Result<T, AppError>;

#[instrument(skip(state, request), fields(card = %request.card))]
pub async fn handle_post(
    State(state): State<crate::state::State>,
    Json(request): Json<RegisterUserRequest>,
) -> AppResult<(StatusCode, Json<UserDataResponse>)> {
    info!(card = %request.card, display_name = %request.display_name, "Register user request received");
    let user_data = state.usecases.user.register(request.into()).await?;
    info!(user_id = %user_data.id, "User registered successfully");
    Ok((StatusCode::CREATED, Json(user_data.into())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::{self, Body},
        http::Request,
    };
    use chrono::NaiveDate;
    use domain::{
        entity::{rating::Rating, user::User},
        repository::{MockRepositories, user::UserRepositoryError},
    };
    use serde_json::json;
    use tower::ServiceExt;

    fn test_router(user_repo: domain::repository::user::MockUserRepository) -> Router {
        let config = crate::config::Config::default();
        let repositories = MockRepositories { user: user_repo };
        let state = crate::state::State::new(config, repositories);
        super::super::create_app(state)
    }

    #[tokio::test]
    async fn handle_post_returns_created_on_success() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_create()
            .withf(|user| user.card() == "CARD-500" && user.display_name() == "Dave")
            .returning(|_| {
                Box::pin(async {
                    let created_at = NaiveDate::from_ymd_opt(2025, 10, 21)
                        .unwrap()
                        .and_hms_opt(15, 0, 0)
                        .unwrap();
                    Ok(User::new(
                        "550e8400-e29b-41d4-a716-446655440000".to_owned(),
                        "CARD-500".to_owned(),
                        "Dave".to_owned(),
                        Rating::new(2500),
                        999,
                        123,
                        false,
                        created_at,
                    ))
                })
            });

        let router = test_router(user_repo);

        let payload = json!({
            "card": "CARD-500",
            "display_name": "Dave"
        });

        let response = router
            .oneshot(
                Request::post("/users")
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert_eq!(response.status(), StatusCode::CREATED);

        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["id"], "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(json["card"], "CARD-500");
        assert_eq!(json["display_name"], "Dave");
        assert_eq!(json["rating"], 2500);
        assert_eq!(json["xp"], 999);
        assert_eq!(json["credits"], 123);
        assert_eq!(json["is_admin"], false);
        assert_eq!(json["created_at"], "2025-10-21 15:00:00");
    }

    #[tokio::test]
    async fn handle_post_maps_repository_conflicts() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo.expect_create().returning(|_| {
            Box::pin(async {
                Err(UserRepositoryError::CardIdAlreadyExists(
                    "CARD-500".to_owned(),
                ))
            })
        });

        let router = test_router(user_repo);

        let payload = json!({
            "card": "CARD-500",
            "display_name": "Eve"
        });

        let response = router
            .oneshot(
                Request::post("/users")
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert_eq!(response.status(), StatusCode::CONFLICT);

        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            json["error"]
                .as_str()
                .unwrap()
                .contains("Card ID already exists")
        );
    }
}
