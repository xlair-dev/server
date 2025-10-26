use domain::repository::user::UserRepositoryError;
use usecase::user::UserUsecaseError;

use crate::error::AppError;

impl From<UserRepositoryError> for AppError {
    fn from(error: UserRepositoryError) -> Self {
        match error {
            UserRepositoryError::CardIdAlreadyExists(_) => AppError {
                status_code: axum::http::StatusCode::CONFLICT,
                message: error.to_string(),
            },
            UserRepositoryError::InternalError(err) => AppError {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message: err.to_string(),
            },
        }
    }
}

impl From<UserUsecaseError> for AppError {
    fn from(error: UserUsecaseError) -> Self {
        match error {
            UserUsecaseError::UserRepositoryError(repo_error) => repo_error.into(),
            UserUsecaseError::NotFound(card) => AppError {
                status_code: axum::http::StatusCode::NOT_FOUND,
                message: format!("User not found for card: {card}"),
            },
            UserUsecaseError::InternalError(err) => AppError {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message: err.to_string(),
            },
        }
    }
}
