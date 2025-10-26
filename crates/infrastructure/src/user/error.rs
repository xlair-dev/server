use anyhow::Error as AnyError;
use domain::repository::user::UserRepositoryError;
use sea_orm::{DbErr, error::SqlErr};

/// Map SeaORM insertion errors into domain-specific variants to preserve invariants.
pub fn convert_user_insert_error(err: DbErr, card: &str) -> UserRepositoryError {
    if matches!(err, DbErr::RecordNotInserted) {
        return UserRepositoryError::CardIdAlreadyExists(card.to_owned());
    }

    if let Some(sql_err) = err.sql_err()
        && matches!(sql_err, SqlErr::UniqueConstraintViolation(_))
    {
        return UserRepositoryError::CardIdAlreadyExists(card.to_owned());
    }

    UserRepositoryError::InternalError(AnyError::from(err))
}
