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
) -> AppResult<Response<Body>> {
    let music = state.usecases.music.find_by_id(music_id).await?;
    let key = music.music.jacket_key.ok_or_else(AppError::not_found)?;
    stream_asset(&state, &key, &file_name, "image/png", true).await
}

#[instrument(skip(state), fields(music_id = %music_id))]
pub async fn handle_get_audio(
    State(state): State<crate::state::State>,
    Path((music_id, file_name)): Path<(String, String)>,
) -> AppResult<Response<Body>> {
    let music = state.usecases.music.find_by_id(music_id).await?;
    let key = music.music.music_key.ok_or_else(AppError::not_found)?;
    stream_asset(&state, &key, &file_name, "audio/wav", false).await
}

#[instrument(skip(state), fields(sheet_id = %sheet_id))]
pub async fn handle_get_chart(
    State(state): State<crate::state::State>,
    Path((sheet_id, file_name)): Path<(String, String)>,
) -> AppResult<Response<Body>> {
    let sheet = state.usecases.music.find_by_sheet_id(sheet_id).await?;
    let key = sheet.ok_or_else(AppError::not_found)?;
    stream_asset(&state, &key, &file_name, "text/plain; charset=utf-8", false).await
}

async fn stream_asset(
    state: &crate::state::State,
    key: &str,
    file_name: &str,
    content_type: &str,
    cacheable: bool,
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
    let download = storage.download(key).await.map_err(|error| {
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
