use anyhow::Error as AnyError;
use domain::repository::user::UserRepositoryError;
use sea_orm::{DbErr, error::SqlErr, prelude::Uuid};
use tracing::{debug, error, warn};

/// Map SeaORM insertion errors into domain-specific variants to preserve invariants.
pub fn convert_user_insert_error(err: DbErr, card: &str) -> UserRepositoryError {
    if matches!(err, DbErr::RecordNotInserted) {
        warn!(card = %card, "Card ID already exists");
        return UserRepositoryError::CardIdAlreadyExists(card.to_owned());
    }

    if let Some(sql_err) = err.sql_err()
        && matches!(sql_err, SqlErr::UniqueConstraintViolation(_))
    {
        warn!(card = %card, "Unique constraint violation for card ID");
        return UserRepositoryError::CardIdAlreadyExists(card.to_owned());
    }

    error!(error = %err, "Failed to insert user");
    UserRepositoryError::InternalError(AnyError::from(err))
}

pub fn parse_user_uuid(user_id: &str) -> Result<Uuid, UserRepositoryError> {
    Uuid::parse_str(user_id).map_err(|err| {
        debug!(error = %err, "Failed to parse user id");
        UserRepositoryError::NotFound(user_id.to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{repository::user::UserRepositoryError, testing::user::USER1};
    use sea_orm::RuntimeErr;

    #[test]
    fn convert_user_insert_error_returns_conflict_for_record_not_inserted() {
        let error = DbErr::RecordNotInserted;
        let result = convert_user_insert_error(error, USER1.card);

        match result {
            UserRepositoryError::CardIdAlreadyExists(card) => assert_eq!(card, USER1.card),
            _ => panic!("expected CardIdAlreadyExists"),
        }
    }

    #[test]
    fn convert_user_insert_error_wraps_other_errors() {
        let error = DbErr::Conn(RuntimeErr::Internal("boom".to_owned()));
        let result = convert_user_insert_error(error, USER1.card);

        match result {
            UserRepositoryError::InternalError(inner) => {
                assert!(inner.to_string().contains("Connection Error"));
            }
            _ => panic!("expected InternalError"),
        }
    }
}
