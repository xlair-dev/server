use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use tracing::{info, instrument};

use crate::{
    error::AppError,
    model::user::{FindUserQuery, RegisterUserRequest, UserDataResponse},
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

#[instrument(skip(state, query), fields(card = %query.card))]
pub async fn handle_get(
    State(state): State<crate::state::State>,
    Query(query): Query<FindUserQuery>,
) -> AppResult<Json<UserDataResponse>> {
    info!(card = %query.card, "Get user request received");
    let user_data = state.usecases.user.find_by_card(query.card.clone()).await?;
    info!(user_id = %user_data.id, "User retrieved successfully");
    Ok(Json(user_data.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::{self, Body},
        http::Request,
    };
    use domain::entity::rating::Rating;
    use domain::{
        entity::user::User,
        repository::{MockRepositories, user::UserRepositoryError},
        testing::{
            datetime::timestamp,
            user::{USER1, USER2},
        },
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
            .withf(|user| user.card() == USER2.card && user.display_name() == USER2.display_name)
            .returning(|_| {
                Box::pin(async {
                    Ok(User::new(
                        USER2.id.to_owned(),
                        USER2.card.to_owned(),
                        USER2.display_name.to_owned(),
                        Rating::new(USER2.rating),
                        USER2.xp,
                        USER2.credits,
                        false,
                        timestamp(2025, 10, 21, 15, 0, 0),
                    ))
                })
            });

        let router = test_router(user_repo);

        let payload = json!({
            "card": USER2.card,
            "display_name": USER2.display_name
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
        assert_eq!(json["id"], USER2.id);
        assert_eq!(json["card"], USER2.card);
        assert_eq!(json["display_name"], USER2.display_name);
        assert_eq!(json["rating"], USER2.rating);
        assert_eq!(json["xp"], USER2.xp);
        assert_eq!(json["credits"], USER2.credits);
        assert_eq!(json["is_admin"], false);
        assert_eq!(json["created_at"], "2025-10-21 15:00:00");
    }

    #[tokio::test]
    async fn handle_post_maps_repository_conflicts() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo.expect_create().returning(|_| {
            Box::pin(async {
                Err(UserRepositoryError::CardIdAlreadyExists(
                    USER2.card.to_owned(),
                ))
            })
        });

        let router = test_router(user_repo);

        let payload = json!({
            "card": USER2.card,
            "display_name": USER1.display_name
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

    #[tokio::test]
    async fn handle_get_returns_user_on_success() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_find_by_card()
            .withf(|card| card == USER1.card)
            .returning(|_| {
                let user = USER1.build(timestamp(2025, 10, 21, 15, 0, 0), false);
                Box::pin(async move { Ok(Some(user)) })
            });

        let router = test_router(user_repo);

        let response = router
            .oneshot(
                Request::get("/users?card=CARD-001")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["id"], USER1.id);
        assert_eq!(json["card"], USER1.card);
        assert_eq!(json["display_name"], USER1.display_name);
    }

    #[tokio::test]
    async fn handle_get_returns_not_found_when_missing() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_find_by_card()
            .withf(|card| card == USER2.card)
            .returning(|_| Box::pin(async { Ok(None) }));

        let router = test_router(user_repo);

        let response = router
            .oneshot(
                Request::get("/users?card=CARD-002")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["error"].as_str().unwrap().contains("User not found"));
    }
}
