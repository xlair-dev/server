use axum::{
    Json,
    extract::{Path, State},
};
use tracing::{info, instrument};

use crate::{
    error::AppError,
    model::ranking::{
        RatingRankingResponse, SheetScoreRankingResponse, TotalScoreRankingResponse,
        XpRankingResponse,
    },
};

type AppResult<T> = Result<T, AppError>;

#[instrument(skip(state), fields(sheet_id = %sheet_id))]
pub async fn handle_get_sheet_ranking(
    Path(sheet_id): Path<String>,
    State(state): State<crate::state::State>,
) -> AppResult<Json<SheetScoreRankingResponse>> {
    info!("Sheet ranking request received");
    let ranking = state.usecases.ranking.sheet_high_scores(&sheet_id).await?;
    let entry_count = ranking.entries.len();
    let response = SheetScoreRankingResponse::from(ranking);
    info!(
        sheet_id = %sheet_id,
        entry_count,
        "Sheet ranking computed successfully"
    );
    Ok(Json(response))
}

#[instrument(skip(state))]
pub async fn handle_get_total_ranking(
    State(state): State<crate::state::State>,
) -> AppResult<Json<TotalScoreRankingResponse>> {
    info!("Total score ranking request received");
    let ranking = state.usecases.ranking.total_high_scores().await?;
    let entry_count = ranking.entries.len();
    let response = TotalScoreRankingResponse::from(ranking);
    info!(entry_count, "Total score ranking computed successfully");
    Ok(Json(response))
}

#[instrument(skip(state))]
pub async fn handle_get_rating_ranking(
    State(state): State<crate::state::State>,
) -> AppResult<Json<RatingRankingResponse>> {
    info!("Rating ranking request received");
    let ranking = state.usecases.ranking.rating().await?;
    let entry_count = ranking.entries.len();
    let response = RatingRankingResponse::from(ranking);
    info!(entry_count, "Rating ranking computed successfully");
    Ok(Json(response))
}

#[instrument(skip(state))]
pub async fn handle_get_xp_ranking(
    State(state): State<crate::state::State>,
) -> AppResult<Json<XpRankingResponse>> {
    info!("XP ranking request received");
    let ranking = state.usecases.ranking.xp().await?;
    let entry_count = ranking.entries.len();
    let response = XpRankingResponse::from(ranking);
    info!(entry_count, "XP ranking computed successfully");
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use axum::{Router, body, http::Request};
    use domain::{
        entity::{rating::Rating, user::User},
        repository::{
            MockRepositories,
            music::MockMusicRepository,
            record::{MockRecordRepository, SheetScoreRankingRow, TotalScoreRankingRow},
            user::MockUserRepository,
        },
    };
    use serde_json::Value;
    use tower::ServiceExt;

    fn build_router(user_repo: MockUserRepository, record_repo: MockRecordRepository) -> Router {
        let config = crate::config::Config::default();
        let repositories = MockRepositories {
            user: user_repo,
            record: record_repo,
            music: MockMusicRepository::new(),
        };
        let state = crate::state::State::new(config, repositories);
        super::super::create_app(state)
    }

    fn sample_user(id: &str, display: &str, rating: u32, xp: u32) -> User {
        User::new(
            id.to_owned(),
            format!("CARD-{id}"),
            display.to_owned(),
            Rating::new(rating),
            xp,
            0,
            true,
            false,
            chrono::Utc::now(),
        )
    }

    #[tokio::test]
    async fn handle_get_sheet_ranking_returns_entries() {
        let mut record_repo = MockRecordRepository::new();
        record_repo
            .expect_find_public_high_scores_by_sheet()
            .returning(|sheet_id, limit| {
                assert_eq!(sheet_id, "sheet-123");
                assert_eq!(limit, 20);
                Box::pin(async {
                    Ok(vec![SheetScoreRankingRow::new(
                        "user-1".to_owned(),
                        "Alice".to_owned(),
                        987_654,
                    )])
                })
            });

        let router = build_router(MockUserRepository::new(), record_repo);
        let response = router
            .oneshot(
                Request::get("/rankings/sheets/sheet-123")
                    .body(body::Body::empty())
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert!(response.status().is_success());
        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["sheetId"], "sheet-123");
        assert_eq!(json["entries"][0]["userId"], "user-1");
        assert_eq!(json["entries"][0]["score"], 987_654);
    }

    #[tokio::test]
    async fn handle_get_total_ranking_returns_entries() {
        let mut record_repo = MockRecordRepository::new();
        record_repo
            .expect_find_public_total_score_ranking()
            .returning(|limit| {
                assert_eq!(limit, 20);
                Box::pin(async {
                    Ok(vec![TotalScoreRankingRow::new(
                        "user-2".to_owned(),
                        "Bob".to_owned(),
                        1_234_567,
                    )])
                })
            });

        let router = build_router(MockUserRepository::new(), record_repo);
        let response = router
            .oneshot(
                Request::get("/rankings/total-score")
                    .body(body::Body::empty())
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert!(response.status().is_success());
        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["entries"][0]["userId"], "user-2");
        assert_eq!(json["entries"][0]["totalScore"], 1_234_567);
    }

    #[tokio::test]
    async fn handle_get_rating_ranking_returns_entries() {
        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_find_public_top_by_rating()
            .returning(|limit| {
                assert_eq!(limit, 20);
                Box::pin(async { Ok(vec![sample_user("user-3", "Carol", 1900, 50)]) })
            });

        let router = build_router(user_repo, MockRecordRepository::new());
        let response = router
            .oneshot(
                Request::get("/rankings/rating")
                    .body(body::Body::empty())
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert!(response.status().is_success());
        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["entries"][0]["userId"], "user-3");
        assert_eq!(json["entries"][0]["rating"], 1900);
    }

    #[tokio::test]
    async fn handle_get_xp_ranking_returns_entries() {
        let mut user_repo = MockUserRepository::new();
        user_repo.expect_find_public_top_by_xp().returning(|limit| {
            assert_eq!(limit, 20);
            Box::pin(async { Ok(vec![sample_user("user-4", "Dave", 1800, 123)]) })
        });

        let router = build_router(user_repo, MockRecordRepository::new());
        let response = router
            .oneshot(
                Request::get("/rankings/xp")
                    .body(body::Body::empty())
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert!(response.status().is_success());
        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["entries"][0]["userId"], "user-4");
        assert_eq!(json["entries"][0]["xp"], 123);
    }
}
