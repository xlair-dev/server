use domain::repository::{record::RecordRepositoryError, user::UserRepositoryError};
use usecase::user::UserUsecaseError;

use crate::error::AppError;

impl From<UserRepositoryError> for AppError {
    fn from(error: UserRepositoryError) -> Self {
        match error {
            UserRepositoryError::CardIdAlreadyExists(_) => AppError {
                status_code: axum::http::StatusCode::CONFLICT,
                message: error.to_string(),
            },
            UserRepositoryError::NotFound(id) => AppError {
                status_code: axum::http::StatusCode::NOT_FOUND,
                message: format!("User not found: {id}"),
            },
            UserRepositoryError::InternalError(err) => AppError {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message: err.to_string(),
            },
        }
    }
}

impl From<RecordRepositoryError> for AppError {
    fn from(error: RecordRepositoryError) -> Self {
        match error {
            RecordRepositoryError::UserNotFound(id) => AppError {
                status_code: axum::http::StatusCode::NOT_FOUND,
                message: format!("User not found: {id}"),
            },
            RecordRepositoryError::SheetNotFound(id) => AppError {
                status_code: axum::http::StatusCode::NOT_FOUND,
                message: format!("Sheet not found: {id}"),
            },
            RecordRepositoryError::InternalError(err) => AppError {
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
            UserUsecaseError::NotFoundByCard { card } => AppError {
                status_code: axum::http::StatusCode::NOT_FOUND,
                message: format!("User not found for card: {card}"),
            },
            UserUsecaseError::NotFoundById { user_id } => AppError {
                status_code: axum::http::StatusCode::NOT_FOUND,
                message: format!("User not found for id: {user_id}"),
            },
            UserUsecaseError::RecordRepositoryError(repo_error) => repo_error.into(),
            UserUsecaseError::InternalError(err) => AppError {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message: err.to_string(),
            },
        }
    }
}
