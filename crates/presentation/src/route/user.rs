use axum::extract::State;

pub async fn handle_post(State(state): State<crate::state::State>) -> String {
    "User created successfully".to_string()
}
