use axum::{
    Json,
    extract::{Query, State},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use domain::repository::music::MusicListCursor;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{error::AppError, model::sync::SyncItemResponse};

const DEFAULT_PAGE_LIMIT: u64 = 50;
const MAX_PAGE_LIMIT: u64 = 100;

#[derive(Deserialize)]
pub struct MusicListQuery {
    pub cursor: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicListResponse {
    pub items: Vec<SyncItemResponse>,
    pub next_cursor: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct CursorPayload {
    registration_date: String,
    id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbSynchronizationResponse {
    pub updated_users: u64,
    pub updated_ratings: u64,
}

pub async fn handle_list_musics(
    State(state): State<crate::state::State>,
    Query(query): Query<MusicListQuery>,
) -> Result<Json<MusicListResponse>, AppError> {
    let limit = query.limit.unwrap_or(DEFAULT_PAGE_LIMIT);
    if !(1..=MAX_PAGE_LIMIT).contains(&limit) {
        return Err(AppError::bad_request(format!(
            "limit must be between 1 and {MAX_PAGE_LIMIT}"
        )));
    }

    let cursor = query.cursor.as_deref().map(decode_cursor).transpose()?;
    let page = state.usecases.music.list_page(cursor, limit).await?;
    let next_cursor = page.next_cursor.map(encode_cursor).transpose()?;
    let items = page.items.into_iter().map(SyncItemResponse::from).collect();

    Ok(Json(MusicListResponse { items, next_cursor }))
}

fn encode_cursor(cursor: MusicListCursor) -> Result<String, AppError> {
    let payload = CursorPayload {
        registration_date: cursor.registration_date.to_rfc3339(),
        id: cursor.id,
    };
    let bytes = serde_json::to_vec(&payload).map_err(|error| {
        AppError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            error.to_string(),
        )
    })?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_cursor(value: &str) -> Result<MusicListCursor, AppError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| AppError::bad_request("cursor is invalid"))?;
    let payload: CursorPayload =
        serde_json::from_slice(&bytes).map_err(|_| AppError::bad_request("cursor is invalid"))?;
    let registration_date = DateTime::parse_from_rfc3339(&payload.registration_date)
        .map_err(|_| AppError::bad_request("cursor is invalid"))?
        .with_timezone(&Utc);
    if uuid::Uuid::parse_str(&payload.id).is_err() {
        return Err(AppError::bad_request("cursor is invalid"));
    }
    Ok(MusicListCursor {
        registration_date,
        id: payload.id,
    })
}

pub async fn handle_db_synchronization(
    State(state): State<crate::state::State>,
) -> Result<Json<DbSynchronizationResponse>, AppError> {
    let result = state.usecases.user.synchronize_db().await?;
    info!(
        updated_users = result.updated_users,
        "Admin database synchronization completed"
    );
    Ok(Json(DbSynchronizationResponse {
        updated_users: result.updated_users,
        updated_ratings: result.updated_ratings,
    }))
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn cursor_round_trip_preserves_ordering_key() {
        let cursor = MusicListCursor {
            registration_date: Utc.with_ymd_and_hms(2025, 10, 1, 12, 0, 0).unwrap(),
            id: "00000000-0000-0000-0000-000000000001".to_owned(),
        };

        let encoded = encode_cursor(cursor.clone()).unwrap();
        assert_eq!(decode_cursor(&encoded).unwrap(), cursor);
    }

    #[test]
    fn invalid_cursor_is_rejected() {
        assert_eq!(
            decode_cursor("not-a-cursor").unwrap_err().status_code,
            axum::http::StatusCode::BAD_REQUEST
        );
    }
}
