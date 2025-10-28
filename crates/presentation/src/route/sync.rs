use axum::{Json, extract::State};
use tracing::{info, instrument};

use crate::{error::AppError, model::sync::SyncItemResponse};

type AppResult<T> = Result<T, AppError>;

#[instrument(skip(state))]
pub async fn handle_get(
    State(state): State<crate::state::State>,
) -> AppResult<Json<Vec<SyncItemResponse>>> {
    info!("Sync metadata request received");
    let musics = state.usecases.music.list_all().await?;
    let response: Vec<SyncItemResponse> = musics.into_iter().map(SyncItemResponse::from).collect();
    info!(count = response.len(), "Sync metadata response prepared");
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use axum::{Router, body, http::Request};
    use chrono::{TimeZone, Utc};
    use domain::{
        entity::{difficulty::Difficulty, genre::Genre, level::Level, music::Music, sheet::Sheet},
        repository::{
            MockRepositories,
            music::{MockMusicRepository, MusicWithSheets},
            record::MockRecordRepository,
            user::MockUserRepository,
        },
    };
    use serde_json::Value;
    use tower::ServiceExt;

    fn build_router(music_repo: MockMusicRepository) -> Router {
        let config = crate::config::Config::default();
        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: MockRecordRepository::new(),
            music: music_repo,
        };
        let state = crate::state::State::new(config, repositories);
        super::super::create_app(state)
    }

    #[tokio::test]
    async fn handle_get_returns_music() {
        let mut music_repo = MockMusicRepository::new();
        music_repo.expect_list_with_sheets().returning(|| {
            let music = Music::new(
                "music-1".to_owned(),
                "Song".to_owned(),
                "Artist".to_owned(),
                140.0,
                Genre::ORIGINAL,
                "jackets/song.png".to_owned(),
                Utc.with_ymd_and_hms(2025, 10, 1, 12, 0, 0).unwrap(),
                false,
            );
            let sheet = Sheet::new(
                "sheet-1".to_owned(),
                "music-1".to_owned(),
                Difficulty::Hard,
                Level::new(13, 7).expect("level"),
                "Designer".to_owned(),
            );
            Box::pin(async move { Ok(vec![MusicWithSheets::new(music, vec![sheet])]) })
        });

        let router = build_router(music_repo);
        let response = router
            .oneshot(Request::get("/sync").body(body::Body::empty()).unwrap())
            .await
            .expect("handler should respond");

        assert!(response.status().is_success());
        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json.as_array().unwrap().len(), 1);
        let first = &json[0];
        assert_eq!(first["music"]["id"], "music-1");
        assert_eq!(first["music"]["bpm"], 140.0);
        assert_eq!(first["sheets"].as_array().unwrap().len(), 1);
        assert_eq!(first["sheets"][0]["difficulty"], "hard");
        assert_eq!(first["sheets"][0]["level"], 13.7);
    }
}
