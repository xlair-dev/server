use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use tracing::{info, instrument};
use usecase::model::user::UserRecordSubmissionDto;

use crate::{
    error::AppError,
    model::user::{
        CreditsIncrementResponse, FindUserQuery, RegisterUserRequest, UpdateUserRequest,
        UserDataResponse, UserRecordRequest, UserRecordResponse,
    },
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

#[instrument(skip(state), fields(user_id = %user_id))]
pub async fn handle_increment_credits(
    State(state): State<crate::state::State>,
    Path(user_id): Path<String>,
) -> AppResult<Json<CreditsIncrementResponse>> {
    info!("Increment credits request received");
    let result = state.usecases.user.increment_credits(user_id).await?;
    info!(credits = result.credits, "Credits incremented successfully");
    Ok(Json(result.into()))
}

#[instrument(skip(state), fields(user_id = %user_id))]
pub async fn handle_get_records(
    State(state): State<crate::state::State>,
    Path(user_id): Path<String>,
) -> AppResult<Json<Vec<UserRecordResponse>>> {
    info!("Get user records request received");
    let records = state.usecases.user.list_records(user_id.clone()).await?;
    info!(count = records.len(), "User records retrieved successfully");
    let response: Vec<UserRecordResponse> =
        records.into_iter().map(UserRecordResponse::from).collect();
    Ok(Json(response))
}

#[instrument(skip(state, request), fields(user_id = %user_id))]
pub async fn handle_update_user(
    State(state): State<crate::state::State>,
    Path(user_id): Path<String>,
    Json(request): Json<UpdateUserRequest>,
) -> AppResult<Json<UserDataResponse>> {
    info!("Update user request received");
    let user_data = state
        .usecases
        .user
        .update_user(user_id, request.into())
        .await?;
    info!(user_id = %user_data.id, "User updated successfully");
    Ok(Json(user_data.into()))
}

#[instrument(skip(state, payload), fields(user_id = %user_id))]
pub async fn handle_post_records(
    State(state): State<crate::state::State>,
    Path(user_id): Path<String>,
    Json(payload): Json<Vec<UserRecordRequest>>,
) -> AppResult<(StatusCode, Json<Vec<UserRecordResponse>>)> {
    info!(
        count = payload.len(),
        "Submit user records request received"
    );

    for request in &payload {
        if request.user_id != user_id {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "userId in payload must match path parameter".to_owned(),
            ));
        }
    }

    let mut submissions = Vec::with_capacity(payload.len());
    for request in payload {
        let dto = UserRecordSubmissionDto::try_from(request)
            .map_err(|err| AppError::new(StatusCode::BAD_REQUEST, err))?;
        submissions.push(dto);
    }

    let records = state
        .usecases
        .user
        .submit_records(user_id.clone(), submissions)
        .await?;
    let response: Vec<UserRecordResponse> =
        records.into_iter().map(UserRecordResponse::from).collect();
    info!(
        count = response.len(),
        "User records persisted successfully"
    );
    Ok((StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::{self, Body},
        http::Request,
    };
    use domain::{
        entity::{clear_type::ClearType, level::Level, rating::Rating, record::Record, user::User},
        repository::{
            MockRepositories,
            music::MockMusicRepository,
            record::{MockRecordRepository, RecordWithMetadata},
            user::UserRepositoryError,
        },
        testing::{
            datetime::timestamp,
            user::{USER1, USER2},
        },
    };
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;

    fn test_router(
        user_repo: domain::repository::user::MockUserRepository,
        record_repo: MockRecordRepository,
    ) -> Router {
        let config = crate::config::Config::default();
        let repositories = MockRepositories {
            user: user_repo,
            record: record_repo,
            music: MockMusicRepository::new(),
        };
        let state = crate::state::State::new(config, repositories);
        super::super::create_app(state)
    }

    fn sample_timestamp() -> chrono::DateTime<chrono::Utc> {
        chrono::NaiveDate::from_ymd_opt(2025, 10, 26)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc()
    }

    #[tokio::test]
    async fn handle_post_returns_created_on_success() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_create()
            .withf(|user| user.card() == USER2.card && user.display_name() == USER2.display_name && !user.is_public())
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
                        false,
                        timestamp(2025, 10, 21, 15, 0, 0),
                    ))
                })
            });

        let router = test_router(user_repo, MockRecordRepository::new());

        let payload = json!({
            "card": USER2.card,
            "displayName": USER2.display_name,
            "isPublic": false
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
        assert_eq!(json["displayName"], USER2.display_name);
        assert_eq!(json["rating"], USER2.rating);
        assert_eq!(json["xp"], USER2.xp);
        assert_eq!(json["credits"], USER2.credits);
        assert_eq!(json["isPublic"], false);
        assert_eq!(json["isAdmin"], false);
        assert_eq!(json["createdAt"], "2025-10-21T15:00:00+00:00");
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

        let router = test_router(user_repo, MockRecordRepository::new());

        let payload = json!({
            "card": USER2.card,
            "displayName": USER1.display_name,
            "isPublic": false
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
                let user = USER1.build(false, false, timestamp(2025, 10, 21, 15, 0, 0));
                Box::pin(async move { Ok(Some(user)) })
            });

        let router = test_router(user_repo, MockRecordRepository::new());

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
        assert_eq!(json["displayName"], USER1.display_name);
    }

    #[tokio::test]
    async fn handle_get_returns_not_found_when_missing() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_find_by_card()
            .withf(|card| card == USER2.card)
            .returning(|_| Box::pin(async { Ok(None) }));

        let router = test_router(user_repo, MockRecordRepository::new());

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

    #[tokio::test]
    async fn handle_increment_credits_returns_current_value() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_increment_credits()
            .withf(|user_id| user_id == USER1.id)
            .returning(|_| Box::pin(async { Ok(USER1.credits + 1) }));

        let router = test_router(user_repo, MockRecordRepository::new());

        let response = router
            .oneshot(
                Request::post(format!("/users/{}/credits/increment", USER1.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["credits"], USER1.credits + 1);
    }

    #[tokio::test]
    async fn handle_increment_credits_returns_not_found() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo.expect_increment_credits().returning(|_| {
            Box::pin(async { Err(UserRepositoryError::NotFound("missing".to_owned())) })
        });

        let router = test_router(user_repo, MockRecordRepository::new());

        let response = router
            .oneshot(
                Request::post("/users/missing/credits/increment")
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

    #[tokio::test]
    async fn handle_get_records_returns_data() {
        let mut record_repo = MockRecordRepository::new();
        record_repo
            .expect_find_by_user_id()
            .withf(|user_id| user_id == USER1.id)
            .returning(|_| {
                Box::pin(async move {
                    Ok(vec![Record::new(
                        "record-1".to_owned(),
                        USER1.id.to_owned(),
                        "sheet-1".to_owned(),
                        1_000_000,
                        ClearType::Clear,
                        5,
                        chrono::NaiveDate::from_ymd_opt(2025, 10, 26)
                            .unwrap()
                            .and_hms_opt(12, 0, 0)
                            .unwrap()
                            .and_utc(),
                    )])
                })
            });

        let router = test_router(
            domain::repository::user::MockUserRepository::new(),
            record_repo,
        );

        let response = router
            .oneshot(
                Request::get(format!("/users/{}/records", USER1.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0]["id"], "record-1");
        assert_eq!(json[0]["clearType"], "clear");
    }

    #[tokio::test]
    async fn handle_get_records_returns_not_found() {
        let mut record_repo = MockRecordRepository::new();
        record_repo.expect_find_by_user_id().returning(|_| {
            Box::pin(async {
                Err(
                    domain::repository::record::RecordRepositoryError::UserNotFound(
                        "missing".to_owned(),
                    ),
                )
            })
        });

        let router = test_router(
            domain::repository::user::MockUserRepository::new(),
            record_repo,
        );

        let response = router
            .oneshot(
                Request::get("/users/missing/records")
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

    #[tokio::test]
    async fn handle_post_records_returns_created() {
        let mut record_repo = MockRecordRepository::new();
        record_repo
            .expect_find_by_user_id_and_sheet_ids()
            .withf(|user_id, sheet_ids| {
                user_id == USER1.id && sheet_ids.len() == 1 && sheet_ids[0] == "sheet-1"
            })
            .returning(|_, _| Box::pin(async { Ok(Vec::new()) }));
        record_repo
            .expect_insert()
            .withf(|record| record.user_id() == USER1.id && record.sheet_id() == "sheet-1")
            .returning(|record| {
                Box::pin(async move {
                    Ok(Record::new(
                        record.id().to_owned(),
                        record.user_id().to_owned(),
                        record.sheet_id().to_owned(),
                        *record.score(),
                        *record.clear_type(),
                        *record.play_count(),
                        sample_timestamp(),
                    ))
                })
            });
        record_repo
            .expect_find_with_metadata_by_user_id()
            .withf(|user_id| user_id == USER1.id)
            .returning(|_| {
                Box::pin(async move {
                    let level = Level::new(13, 7).expect("valid level");
                    let record = Record::new(
                        "record-1".to_owned(),
                        USER1.id.to_owned(),
                        "sheet-1".to_owned(),
                        1_000_000,
                        ClearType::FullCombo,
                        1,
                        sample_timestamp(),
                    );
                    Ok(vec![RecordWithMetadata::new(record, level, false)])
                })
            });

        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_find_by_id()
            .withf(|user_id| user_id == USER1.id)
            .returning(|_| {
                let user = User::new(
                    USER1.id.to_owned(),
                    USER1.card.to_owned(),
                    USER1.display_name.to_owned(),
                    Rating::new(USER1.rating),
                    USER1.xp,
                    USER1.credits,
                    false,
                    false,
                    timestamp(2025, 10, 21, 15, 0, 0),
                );
                Box::pin(async move { Ok(Some(user)) })
            });
        user_repo
            .expect_save()
            .withf(|user| {
                user.id() == USER1.id
                    && user.rating().value() == 1470
                    && *user.xp() == USER1.xp + 100
            })
            .returning(|user| Box::pin(async move { Ok(user) }));

        let router = test_router(user_repo, record_repo);

        let payload = json!([{
            "userId": USER1.id,
            "sheetId": "sheet-1",
            "score": 1_000_000,
            "clearType": "fullcombo"
        }]);

        let response = router
            .oneshot(
                Request::post(format!("/users/{}/records", USER1.id))
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert_eq!(response.status(), StatusCode::CREATED);

        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0]["sheetId"], "sheet-1");
        assert_eq!(json[0]["score"], 1_000_000);
    }

    #[tokio::test]
    async fn handle_post_records_validates_user_id() {
        let router = test_router(
            domain::repository::user::MockUserRepository::new(),
            MockRecordRepository::new(),
        );

        let payload = json!([{
            "userId": "someone-else",
            "sheetId": "sheet-1",
            "score": 1_000_000,
            "clearType": "fullcombo"
        }]);

        let response = router
            .oneshot(
                Request::post(format!("/users/{}/records", USER1.id))
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["error"].as_str().unwrap().contains("must match"));
    }

    #[tokio::test]
    async fn handle_post_records_rejects_invalid_clear_type() {
        let router = test_router(
            domain::repository::user::MockUserRepository::new(),
            MockRecordRepository::new(),
        );

        let payload = json!([{
            "userId": USER1.id,
            "sheetId": "sheet-1",
            "score": 900_000,
            "clearType": "unknown"
        }]);

        let response = router
            .oneshot(
                Request::post(format!("/users/{}/records", USER1.id))
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            json["error"]
                .as_str()
                .unwrap()
                .contains("Unsupported clear type")
        );
    }
}
