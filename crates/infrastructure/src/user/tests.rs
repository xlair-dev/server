use super::error::convert_user_insert_error;
use domain::{repository::user::UserRepositoryError, testing::user::USER1};
use sea_orm::{DbErr, RuntimeErr};

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
