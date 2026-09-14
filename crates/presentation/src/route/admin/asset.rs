use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
};
use tracing::{info, instrument};
use usecase::asset::{AssetUpload, AssetUploadError};

use crate::{error::AppError, model::sync::SyncItemResponse};

type AppResult<T> = Result<T, AppError>;

#[instrument(skip(state, headers, body), fields(music_id = %music_id))]
pub async fn upload_jacket(
    State(state): State<crate::state::State>,
    Path(music_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<SyncItemResponse>> {
    let content_type = content_type(&headers, "jacket")?;
    let jacket = AssetUpload::jacket(content_type, body.to_vec()).map_err(map_jacket_error)?;
    let music = state
        .usecases
        .music
        .upload_jacket(storage(&state)?, music_id.clone(), jacket)
        .await?;
    info!(music_id = %music_id, "Admin music jacket uploaded");
    Ok(Json(SyncItemResponse::from(music)))
}

#[instrument(skip(state), fields(music_id = %music_id))]
pub async fn delete_jacket(
    State(state): State<crate::state::State>,
    Path(music_id): Path<String>,
) -> AppResult<Json<SyncItemResponse>> {
    let music = state
        .usecases
        .music
        .delete_jacket(storage(&state)?, music_id.clone())
        .await?;
    info!(music_id = %music_id, "Admin music jacket deleted");
    Ok(Json(SyncItemResponse::from(music)))
}

#[instrument(skip(state, headers, body), fields(music_id = %music_id))]
pub async fn upload_audio(
    State(state): State<crate::state::State>,
    Path(music_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<SyncItemResponse>> {
    let audio = AssetUpload::audio(content_type(&headers, "audio")?, body.to_vec())
        .map_err(map_audio_error)?;
    let music = state
        .usecases
        .music
        .upload_audio(storage(&state)?, music_id.clone(), audio)
        .await?;
    info!(music_id = %music_id, "Admin music audio uploaded");
    Ok(Json(SyncItemResponse::from(music)))
}

#[instrument(skip(state), fields(music_id = %music_id))]
pub async fn delete_audio(
    State(state): State<crate::state::State>,
    Path(music_id): Path<String>,
) -> AppResult<Json<SyncItemResponse>> {
    let music = state
        .usecases
        .music
        .delete_audio(storage(&state)?, music_id.clone())
        .await?;
    info!(music_id = %music_id, "Admin music audio deleted");
    Ok(Json(SyncItemResponse::from(music)))
}

#[instrument(skip(state, headers, body), fields(sheet_id = %sheet_id))]
pub async fn upload_chart(
    State(state): State<crate::state::State>,
    Path(sheet_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<SyncItemResponse>> {
    let chart =
        AssetUpload::chart(chart_file_name(&headers)?, body.to_vec()).map_err(map_chart_error)?;
    let music = state
        .usecases
        .music
        .upload_chart(storage(&state)?, sheet_id.clone(), chart)
        .await?;
    info!(sheet_id = %sheet_id, "Admin sheet chart uploaded");
    Ok(Json(SyncItemResponse::from(music)))
}

#[instrument(skip(state), fields(sheet_id = %sheet_id))]
pub async fn delete_chart(
    State(state): State<crate::state::State>,
    Path(sheet_id): Path<String>,
) -> AppResult<Json<SyncItemResponse>> {
    let music = state
        .usecases
        .music
        .delete_chart(storage(&state)?, sheet_id.clone())
        .await?;
    info!(sheet_id = %sheet_id, "Admin sheet chart deleted");
    Ok(Json(SyncItemResponse::from(music)))
}

fn storage(state: &crate::state::State) -> AppResult<&dyn usecase::asset::AssetStorage> {
    state.asset_storage.as_deref().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Asset storage is not configured".to_owned(),
        )
    })
}

fn content_type<'a>(headers: &'a HeaderMap, asset: &str) -> AppResult<&'a str> {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::bad_request(format!("{asset} content type is required")))
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
