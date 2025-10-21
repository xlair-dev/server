use anyhow::Error as AnyError;
use domain::{
    entity::user::User,
    repository::user::{UserRepository, UserRepositoryError},
};
use sea_orm::{ActiveModelTrait, DbConn, DbErr, error::SqlErr};
use tracing::{debug, error, info, instrument};

use crate::entities;

pub struct UserRepositoryImpl {
    db: DbConn,
}

impl UserRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

impl UserRepository for UserRepositoryImpl {
    #[instrument(skip(self, user), fields(card = %user.card()))]
    async fn create(&self, user: User) -> Result<User, UserRepositoryError> {
        debug!("Persisting user via SeaORM");
        let card_id = user.card().to_owned();
        let db_user: entities::users::ActiveModel = user.into();

        let db_user_model = db_user.insert(&self.db).await.map_err(|err| {
            error!(error = %err, "Failed to insert user");
            convert_user_insert_error(err, &card_id)
        })?;

        info!(user_id = %db_user_model.id, "User persisted by repository");
        Ok(db_user_model.into())
    }
}

/// Convert SeaORM errors emitted during user insertion into domain-specific errors.
///
/// PostgreSQL signals a unique key violation via SQLSTATE `23505`, which SeaORM exposes as
/// `SqlErr::UniqueConstraintViolation`. The `users` table relies on a unique index for the
/// `card` column (see `m20251007_000001_create_users_table`), so we convert that condition into a
/// `UserRepositoryError::CardIdAlreadyExists` to keep the domain invariant explicit.
fn convert_user_insert_error(err: DbErr, card: &str) -> UserRepositoryError {
    if matches!(err, DbErr::RecordNotInserted) {
        return UserRepositoryError::CardIdAlreadyExists(card.to_owned());
    }

    if let Some(sql_err) = err.sql_err() {
        if matches!(sql_err, SqlErr::UniqueConstraintViolation(_)) {
            return UserRepositoryError::CardIdAlreadyExists(card.to_owned());
        }
    }

    UserRepositoryError::InternalError(AnyError::from(err))
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
