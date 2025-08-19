use axum::{extract::State, http::StatusCode, Json};

use crate::model::user::{RegisterUserRequest, UserDataResponse};

pub async fn handle_post(
    State(state): State<crate::state::State>,
    Json(req): Json<RegisterUserRequest>,
) -> (StatusCode, Json<UserDataResponse>) {
    match state.usecases.user.register(req.into()) {
        Ok(user_data) => {
            let user_data = user_data.into();
            (StatusCode::CREATED, Json(user_data))
        }
        Err(_) => {
            // TODO: エラーごとに適切なレスポンスを返す
            let user_data = UserDataResponse {
                id: "".to_string(),
                card: req.card,
                display_name: req.display_name,
                credits: 0,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(user_data))
        }
    }
}
