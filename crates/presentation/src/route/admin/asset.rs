use axum::{
    Json,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
};
use serde::Deserialize;
use tracing::{info, instrument};
use usecase::asset::{AssetUpload, AssetUploadError, ChartFormat};

use crate::{error::AppError, model::sync::SyncItemResponse};

type AppResult<T> = Result<T, AppError>;

#[instrument(skip(state), fields(music_id = %music_id))]
pub async fn get_audio(
    State(state): State<crate::state::State>,
    Path((music_id, file_name)): Path<(String, String)>,
    headers: HeaderMap,
) -> AppResult<axum::http::Response<axum::body::Body>> {
    let music = state.usecases.music.find_by_id(music_id).await?;
    let key = music
        .music
        .audio
        .map(|asset| asset.key)
        .ok_or_else(AppError::not_found)?;
    crate::route::asset::stream_asset(
        &state,
        &key,
        &file_name,
        "audio/wav",
        false,
        crate::route::asset::range(&headers)?,
    )
    .await
}

#[instrument(skip(state), fields(sheet_id = %sheet_id))]
pub async fn get_chart(
    State(state): State<crate::state::State>,
    Path((sheet_id, file_name)): Path<(String, String)>,
    headers: HeaderMap,
) -> AppResult<axum::http::Response<axum::body::Body>> {
    let sheet = state.usecases.music.find_by_sheet_id(sheet_id).await?;
    let key = sheet.ok_or_else(AppError::not_found)?;
    crate::route::asset::stream_asset(
        &state,
        &key,
        &file_name,
        "text/plain; charset=utf-8",
        false,
        crate::route::asset::range(&headers)?,
    )
    .await
}

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

#[instrument(skip(state, query, body), fields(sheet_id = %sheet_id))]
pub async fn upload_chart(
    State(state): State<crate::state::State>,
    Path(sheet_id): Path<String>,
    Query(query): Query<ChartUploadQuery>,
    body: Bytes,
) -> AppResult<Json<SyncItemResponse>> {
    let format = query
        .format
        .parse::<ChartFormat>()
        .map_err(|_| AppError::bad_request("chart format must be ugc or sus"))?;
    let chart = AssetUpload::chart(format, body.to_vec()).map_err(map_chart_error)?;
    let music = state
        .usecases
        .music
        .upload_chart(storage(&state)?, sheet_id.clone(), chart)
        .await?;
    info!(sheet_id = %sheet_id, "Admin sheet chart uploaded");
    Ok(Json(SyncItemResponse::from(music)))
}

#[derive(Deserialize)]
pub struct ChartUploadQuery {
    format: String,
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

fn map_jacket_error(error: AssetUploadError) -> AppError {
    match error {
        AssetUploadError::UnsupportedContentType => {
            AppError::bad_request("jacket must be JPEG, PNG, or WebP")
        }
        AssetUploadError::TooLarge => AppError::bad_request("jacket exceeds 20 MiB"),
        AssetUploadError::InvalidData => AppError::bad_request("jacket image is invalid"),
    }
}

fn map_audio_error(error: AssetUploadError) -> AppError {
    match error {
        AssetUploadError::UnsupportedContentType => AppError::bad_request("audio must be WAV"),
        AssetUploadError::TooLarge => AppError::bad_request("audio exceeds 60 MiB"),
        AssetUploadError::InvalidData => AppError::bad_request("audio is invalid"),
    }
}

fn map_chart_error(error: AssetUploadError) -> AppError {
    match error {
        AssetUploadError::TooLarge => AppError::bad_request("chart exceeds 20 MiB"),
        AssetUploadError::UnsupportedContentType | AssetUploadError::InvalidData => {
            AppError::bad_request("chart is invalid")
        }
    }
}
