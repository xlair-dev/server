pub struct AppError {
    pub status_code: axum::http::StatusCode,
    pub message: String,
}

impl AppError {
    pub fn new(status_code: axum::http::StatusCode, message: String) -> Self {
        Self {
            status_code,
            message,
        }
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let body = axum::Json(serde_json::json!({
            "error": self.message,
        }));
        (self.status_code, body).into_response()
    }
}
