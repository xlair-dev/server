use axum::{
    Json,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use domain::repository::music::MusicListCursor;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use usecase::asset::{AssetUpload, AssetUploadError};

use crate::{
    error::AppError,
    model::{
        admin::{
            CreateMusicRequest, DbSynchronizationResponse, MusicListQuery, MusicListResponse,
            UpdateMusicRequest,
        },
        sync::SyncItemResponse,
    },
};

const DEFAULT_PAGE_LIMIT: u64 = 50;
const MAX_PAGE_LIMIT: u64 = 100;

type AppResult<T> = Result<T, AppError>;

#[derive(Deserialize, Serialize)]
struct CursorPayload {
    registration_date: String,
    id: String,
}

#[instrument(skip(state, query))]
pub async fn handle_list_musics(
    State(state): State<crate::state::State>,
    Query(query): Query<MusicListQuery>,
) -> AppResult<Json<MusicListResponse>> {
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

    info!(limit, "Admin music list retrieved");
    Ok(Json(MusicListResponse { items, next_cursor }))
}

#[instrument(skip(state), fields(music_id = %music_id))]
pub async fn handle_get_music(
    State(state): State<crate::state::State>,
    Path(music_id): Path<String>,
) -> AppResult<Json<SyncItemResponse>> {
    if uuid::Uuid::parse_str(&music_id).is_err() {
        return Err(AppError::bad_request("music id is invalid"));
    }
    let music = state.usecases.music.find_by_id(music_id).await?;
    Ok(Json(SyncItemResponse::from(music)))
}

#[instrument(skip(state, request))]
pub async fn handle_create_music(
    State(state): State<crate::state::State>,
    Json(request): Json<CreateMusicRequest>,
) -> AppResult<(StatusCode, Json<SyncItemResponse>)> {
    let music = state
        .usecases
        .music
        .create(request.try_into_input()?)
        .await?;
    info!(music_id = %music.music.id, "Admin music created");
    Ok((StatusCode::CREATED, Json(SyncItemResponse::from(music))))
}

#[instrument(skip(state, request), fields(music_id = %music_id))]
pub async fn handle_update_music(
    State(state): State<crate::state::State>,
    Path(music_id): Path<String>,
    Json(request): Json<UpdateMusicRequest>,
) -> AppResult<Json<SyncItemResponse>> {
    if uuid::Uuid::parse_str(&music_id).is_err() {
        return Err(AppError::bad_request("music id is invalid"));
    }
    let music = state
        .usecases
        .music
        .update(music_id.clone(), request.try_into()?)
        .await?;
    info!(music_id = %music_id, "Admin music updated");
    Ok(Json(SyncItemResponse::from(music)))
}

#[instrument(skip(state, headers, body), fields(music_id = %music_id))]
pub async fn handle_upload_jacket(
    State(state): State<crate::state::State>,
    Path(music_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<SyncItemResponse>> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::bad_request("jacket content type is required"))?;
    let jacket = AssetUpload::jacket(content_type, body.to_vec()).map_err(map_jacket_error)?;
    let storage = state.asset_storage.as_ref().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "jacket storage is not configured".to_owned(),
        )
    })?;
    let music = state
        .usecases
        .music
        .upload_jacket(storage.as_ref(), music_id.clone(), jacket)
        .await?;
    info!(music_id = %music_id, "Admin music jacket uploaded");
    Ok(Json(SyncItemResponse::from(music)))
}

fn map_jacket_error(error: AssetUploadError) -> AppError {
    match error {
        AssetUploadError::UnsupportedContentType => {
            AppError::bad_request("jacket must be JPEG, PNG, or WebP")
        }
        AssetUploadError::TooLarge => AppError::bad_request("jacket exceeds 5 MiB"),
        AssetUploadError::InvalidData => AppError::bad_request("jacket image is invalid"),
        AssetUploadError::InvalidFileName => AppError::bad_request("jacket file name is invalid"),
    }
}

#[instrument(skip(state), fields(music_id = %music_id))]
pub async fn handle_delete_jacket(
    State(state): State<crate::state::State>,
    Path(music_id): Path<String>,
) -> AppResult<Json<SyncItemResponse>> {
    let storage = state.asset_storage.as_ref().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "jacket storage is not configured".to_owned(),
        )
    })?;
    let music = state
        .usecases
        .music
        .delete_jacket(storage.as_ref(), music_id.clone())
        .await?;
    info!(music_id = %music_id, "Admin music jacket deleted");
    Ok(Json(SyncItemResponse::from(music)))
}

#[instrument(skip(state, headers, body), fields(music_id = %music_id))]
pub async fn handle_upload_audio(
    State(state): State<crate::state::State>,
    Path(music_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<SyncItemResponse>> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::bad_request("audio content type is required"))?;
    let audio = AssetUpload::audio(content_type, body.to_vec()).map_err(map_audio_error)?;
    let storage = asset_storage(&state)?;
    let music = state
        .usecases
        .music
        .upload_audio(storage, music_id.clone(), audio)
        .await?;
    info!(music_id = %music_id, "Admin music audio uploaded");
    Ok(Json(SyncItemResponse::from(music)))
}

#[instrument(skip(state), fields(music_id = %music_id))]
pub async fn handle_delete_audio(
    State(state): State<crate::state::State>,
    Path(music_id): Path<String>,
) -> AppResult<Json<SyncItemResponse>> {
    let storage = asset_storage(&state)?;
    let music = state
        .usecases
        .music
        .delete_audio(storage, music_id.clone())
        .await?;
    info!(music_id = %music_id, "Admin music audio deleted");
    Ok(Json(SyncItemResponse::from(music)))
}

#[instrument(skip(state, headers, body), fields(sheet_id = %sheet_id))]
pub async fn handle_upload_chart(
    State(state): State<crate::state::State>,
    Path(sheet_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<SyncItemResponse>> {
    let file_name = chart_file_name(&headers)?;
    let chart = AssetUpload::chart(file_name, body.to_vec()).map_err(map_chart_error)?;
    let storage = asset_storage(&state)?;
    let music = state
        .usecases
        .music
        .upload_chart(storage, sheet_id.clone(), chart)
        .await?;
    info!(sheet_id = %sheet_id, "Admin sheet chart uploaded");
    Ok(Json(SyncItemResponse::from(music)))
}

#[instrument(skip(state), fields(sheet_id = %sheet_id))]
pub async fn handle_delete_chart(
    State(state): State<crate::state::State>,
    Path(sheet_id): Path<String>,
) -> AppResult<Json<SyncItemResponse>> {
    let storage = asset_storage(&state)?;
    let music = state
        .usecases
        .music
        .delete_chart(storage, sheet_id.clone())
        .await?;
    info!(sheet_id = %sheet_id, "Admin sheet chart deleted");
    Ok(Json(SyncItemResponse::from(music)))
}

fn asset_storage(state: &crate::state::State) -> AppResult<&dyn usecase::asset::AssetStorage> {
    state.asset_storage.as_deref().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Asset storage is not configured".to_owned(),
        )
    })
}

fn chart_file_name(headers: &HeaderMap) -> AppResult<&str> {
    let value = headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::bad_request("chart file name is required"))?;
    value
        .split(';')
        .find_map(|part| part.trim().strip_prefix("filename="))
        .map(|file_name| file_name.trim_matches('"'))
        .filter(|file_name| !file_name.contains('/') && !file_name.contains('\\'))
        .ok_or_else(|| AppError::bad_request("chart file name is invalid"))
}

fn map_audio_error(error: AssetUploadError) -> AppError {
    match error {
        AssetUploadError::UnsupportedContentType => AppError::bad_request("audio must be WAV"),
        AssetUploadError::TooLarge => AppError::bad_request("audio exceeds 30 MiB"),
        AssetUploadError::InvalidData => AppError::bad_request("audio is invalid"),
        AssetUploadError::InvalidFileName => AppError::bad_request("audio file name is invalid"),
    }
}

fn map_chart_error(error: AssetUploadError) -> AppError {
    match error {
        AssetUploadError::TooLarge => AppError::bad_request("chart exceeds 5 MiB"),
        AssetUploadError::InvalidFileName => {
            AppError::bad_request("chart file name must end with .sus")
        }
        AssetUploadError::UnsupportedContentType | AssetUploadError::InvalidData => {
            AppError::bad_request("chart is invalid")
        }
    }
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

#[instrument(skip(state))]
pub async fn handle_db_synchronization(
    State(state): State<crate::state::State>,
) -> AppResult<Json<DbSynchronizationResponse>> {
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
