use axum::{Json, extract::State};
use tracing::{info, instrument};

use crate::{error::AppError, model::statistics::GlobalStatisticsResponse};

type AppResult<T> = Result<T, AppError>;

#[instrument(skip(state))]
pub async fn handle_get_summary(
    State(state): State<crate::state::State>,
) -> AppResult<Json<GlobalStatisticsResponse>> {
    info!("Global statistics summary request received");
    let summary = state.usecases.statistics.summary().await?;
    info!(
        total_users = summary.total_users,
        total_credits = summary.total_credits,
        total_score = summary.total_score,
        "Global statistics summary computed successfully"
    );

    Ok(Json(summary.into()))
}

#[cfg(test)]
mod tests {
    use axum::{Router, body, http::Request};
    use domain::repository::{
        MockRepositories,
        music::MockMusicRepository,
        record::MockRecordRepository,
        user::MockUserRepository,
    };
    use serde_json::Value;
    use tower::ServiceExt;

    fn build_router(
        user_repo: MockUserRepository,
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

    #[tokio::test]
    async fn handle_get_summary_returns_statistics() {
        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_count_all()
            .returning(|| Box::pin(async { Ok(7) }));
        user_repo
            .expect_sum_credits()
            .returning(|| Box::pin(async { Ok(1234) }));

        let mut record_repo = MockRecordRepository::new();
        record_repo
            .expect_sum_scores()
            .returning(|| Box::pin(async { Ok(987_654) }));

        let router = build_router(user_repo, record_repo);
        let response = router
            .oneshot(
                Request::get("/statistics/summary")
                    .body(body::Body::empty())
                    .unwrap(),
            )
            .await
            .expect("handler should respond");

        assert!(response.status().is_success());
        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["totalUsers"], 7);
        assert_eq!(json["totalCredits"], 1234);
        assert_eq!(json["totalScore"], 987_654);
    }
}
