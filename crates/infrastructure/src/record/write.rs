use domain::{entity::record::Record, repository::record::RecordRepositoryError};
use sea_orm::{ActiveModelTrait, DbConn};
use tracing::error;

use crate::record::adapter::{
    active_model_for_insert, active_model_for_update, convert_insert_error, convert_update_error,
};

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
