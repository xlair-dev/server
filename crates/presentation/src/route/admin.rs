use axum::{Json, extract::State};
use serde::Serialize;
use tracing::info;

use crate::error::AppError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbSynchronizationResponse {
    pub updated_users: u64,
    pub updated_ratings: u64,
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
