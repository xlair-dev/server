use axum::{Json, extract::State};
use serde::Serialize;
use tracing::info;

use crate::error::AppError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecalculateRatingsResponse {
    pub updated_users: u64,
}

pub async fn handle_recalculate_ratings(
    State(state): State<crate::state::State>,
) -> Result<Json<RecalculateRatingsResponse>, AppError> {
    let updated_users = state.usecases.user.recalculate_ratings().await?;
    info!(updated_users, "Admin rating recalculation completed");
    Ok(Json(RecalculateRatingsResponse { updated_users }))
}
