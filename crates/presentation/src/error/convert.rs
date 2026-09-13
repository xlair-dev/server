use domain::repository::{
    music::MusicRepositoryError, record::RecordRepositoryError, user::UserRepositoryError,
};
use usecase::{
    music::MusicUsecaseError, ranking::RankingUsecaseError, statistics::StatisticsUsecaseError,
    user::UserUsecaseError,
};

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
                message: internal_error_message(err),
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
                message: internal_error_message(err),
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
                message: internal_error_message(err),
            },
        }
    }
}

impl From<MusicRepositoryError> for AppError {
    fn from(error: MusicRepositoryError) -> Self {
        match error {
            MusicRepositoryError::InvalidLimit(limit) => {
                AppError::bad_request(format!("limit must be greater than 0: {limit}"))
            }
            MusicRepositoryError::NotFound(id) => AppError {
                status_code: axum::http::StatusCode::NOT_FOUND,
                message: format!("Music not found: {id}"),
            },
            MusicRepositoryError::InternalError(err) => AppError {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message: internal_error_message(err),
            },
        }
    }
}

fn internal_error_message(error: anyhow::Error) -> String {
    tracing::error!(error = ?error, "Internal application error");
    "Internal server error".to_owned()
}

impl From<MusicUsecaseError> for AppError {
    fn from(error: MusicUsecaseError) -> Self {
        match error {
            MusicUsecaseError::MusicRepository(err) => err.into(),
            MusicUsecaseError::InvalidInput(message) => AppError::bad_request(message),
            MusicUsecaseError::JacketStorage(error) => {
                tracing::error!(error = ?error, "Jacket storage operation failed");
                AppError::new(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_owned(),
                )
            }
        }
    }
}

impl From<StatisticsUsecaseError> for AppError {
    fn from(error: StatisticsUsecaseError) -> Self {
        match error {
            StatisticsUsecaseError::UserRepository(err) => err.into(),
            StatisticsUsecaseError::RecordRepository(err) => err.into(),
        }
    }
}

impl From<RankingUsecaseError> for AppError {
    fn from(error: RankingUsecaseError) -> Self {
        match error {
            RankingUsecaseError::RecordRepository(err) => err.into(),
            RankingUsecaseError::UserRepository(err) => err.into(),
        }
    }
}
