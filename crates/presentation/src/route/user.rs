use axum::{extract::State, http::StatusCode, Json};
use tracing::{info, instrument};

use crate::{
    error::AppError,
    model::user::{RegisterUserRequest, UserDataResponse},
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
