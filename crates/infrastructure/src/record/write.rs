use anyhow::Error as AnyError;
use domain::{entity::record::Record, repository::record::RecordRepositoryError};
use sea_orm::{ActiveModelTrait, DbConn, DbErr};
use tracing::{debug, error};

use crate::record::adapter::{active_model_for_insert, active_model_for_update};

pub async fn insert_record(db: &DbConn, record: Record) -> Result<Record, RecordRepositoryError> {
    let user_id = record.user_id().to_owned();
    let sheet_id = record.sheet_id().to_owned();
    let active = active_model_for_insert(&record)?;

    match active.insert(db).await {
        Ok(model) => Ok(model.into()),
        Err(err) => {
            error!(error = %err, "Failed to insert record");
            Err(convert_insert_error(err, &user_id, &sheet_id))
        }
    }
}

pub async fn update_record(db: &DbConn, record: Record) -> Result<Record, RecordRepositoryError> {
    let record_id = record.id().to_owned();
    let active = active_model_for_update(&record)?;

    match active.update(db).await {
        Ok(model) => Ok(model.into()),
        Err(err) => {
            error!(error = %err, "Failed to update record");
            Err(convert_update_error(err, &record_id))
        }
    }
}

pub fn convert_insert_error(err: DbErr, user_id: &str, sheet_id: &str) -> RecordRepositoryError {
    let message = err.to_string();
    if message.contains("fk_records_user") {
        return RecordRepositoryError::UserNotFound(user_id.to_owned());
    }

    if message.contains("fk_records_sheet") {
        return RecordRepositoryError::SheetNotFound(sheet_id.to_owned());
    }

    RecordRepositoryError::InternalError(AnyError::from(err))
}

pub fn convert_update_error(err: DbErr, record_id: &str) -> RecordRepositoryError {
    if matches!(err, DbErr::RecordNotUpdated) {
        debug!(record_id = %record_id, "Record not updated; underlying row may be missing");
    }

    RecordRepositoryError::InternalError(AnyError::from(err))
}
