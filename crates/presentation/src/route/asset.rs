use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, Response, StatusCode, header},
};
use tokio_util::io::ReaderStream;
use tracing::instrument;

use crate::error::AppError;

type AppResult<T> = Result<T, AppError>;

#[instrument(skip(state), fields(music_id = %music_id))]
pub async fn handle_get_jacket(
    State(state): State<crate::state::State>,
    Path((music_id, file_name)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> AppResult<Response<Body>> {
    let music = state.usecases.music.find_by_id(music_id).await?;
    let key = music
        .music
        .jacket
        .map(|asset| asset.key)
        .ok_or_else(AppError::not_found)?;
    stream_asset(
        &state,
        &key,
        &file_name,
        "image/png",
        true,
        range(&headers)?,
    )
    .await
}

#[instrument(skip(state), fields(music_id = %music_id))]
pub async fn handle_get_audio(
    State(state): State<crate::state::State>,
    Path((music_id, file_name)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> AppResult<Response<Body>> {
    let music = state.usecases.music.find_by_id(music_id).await?;
    let key = music
        .music
        .audio
        .map(|asset| asset.key)
        .ok_or_else(AppError::not_found)?;
    stream_asset(
        &state,
        &key,
        &file_name,
        "audio/wav",
        false,
        range(&headers)?,
    )
    .await
}

#[instrument(skip(state), fields(sheet_id = %sheet_id))]
pub async fn handle_get_chart(
    State(state): State<crate::state::State>,
    Path((sheet_id, file_name)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> AppResult<Response<Body>> {
    let sheet = state.usecases.music.find_by_sheet_id(sheet_id).await?;
    let key = sheet.ok_or_else(AppError::not_found)?;
    stream_asset(
        &state,
        &key,
        &file_name,
        "text/plain; charset=utf-8",
        false,
        range(&headers)?,
    )
    .await
}

pub(crate) fn range(headers: &axum::http::HeaderMap) -> AppResult<Option<&str>> {
    headers
        .get(axum::http::header::RANGE)
        .map(|value| {
            value
                .to_str()
                .map_err(|_| AppError::bad_request("range header is invalid"))
        })
        .transpose()
}

pub(crate) async fn stream_asset(
    state: &crate::state::State,
    key: &str,
    file_name: &str,
    content_type: &str,
    cacheable: bool,
    range: Option<&str>,
) -> AppResult<Response<Body>> {
    if key.rsplit('/').next() != Some(file_name) {
        return Err(AppError::not_found());
    }
    let storage = state.asset_storage.as_ref().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Asset storage is not configured".to_owned(),
        )
    })?;
    let download = storage.download(key, range).await.map_err(|error| {
        tracing::error!(error = ?error, asset_key = %key, "Asset download failed");
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_owned(),
        )
    })?;
    let mut response = Response::new(Body::from_stream(ReaderStream::new(download.reader)));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type).expect("static content type is valid"),
    );
    response
        .headers_mut()
        .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    response.headers_mut().insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&download.content_length.to_string()).expect("length is valid"),
    );
    if let Some(content_range) = download.content_range {
        response.headers_mut().insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(&content_range).map_err(|_| {
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_owned(),
                )
            })?,
        );
        *response.status_mut() = StatusCode::PARTIAL_CONTENT;
    }
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(if cacheable {
            "public, max-age=31536000, immutable"
        } else {
            "private, no-store"
        }),
    );
    Ok(response)
}
