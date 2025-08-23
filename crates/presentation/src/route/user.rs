use axum::{extract::State, http::StatusCode, Json};

use crate::{
    error::AppError,
    model::user::{RegisterUserRequest, UserDataResponse},
};

type AppResult<T> = Result<T, AppError>;

pub async fn handle_post(
    State(state): State<crate::state::State>,
    Json(req): Json<RegisterUserRequest>,
) -> AppResult<(StatusCode, Json<UserDataResponse>)> {
    let user_data = state.usecases.user.register(req.into())?;
    Ok((StatusCode::CREATED, Json(user_data.into())))
}
