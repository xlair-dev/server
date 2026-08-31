pub mod convert;

#[derive(Debug)]
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

    pub fn not_found() -> Self {
        Self::new(
            axum::http::StatusCode::NOT_FOUND,
            "Resource not found".to_owned(),
        )
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(axum::http::StatusCode::BAD_REQUEST, message.into())
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
