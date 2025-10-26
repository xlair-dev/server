use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Error as AnyError;
use domain::{entity::record::Record, repository::record::RecordRepositoryError};
use sea_orm::{ActiveValue, prelude::Uuid};

use crate::entities::{
    records::ActiveModel as RecordActiveModel, sea_orm_active_enums::ClearType as DbClearType,
};

pub fn active_model_for_insert(
    record: &Record,
) -> Result<RecordActiveModel, RecordRepositoryError> {
    let record_id = if record.id().is_empty() {
        generate_record_uuid(record.user_id(), record.sheet_id())
    } else {
        parse_record_uuid(record.id())?
    };

    let user_uuid = parse_user_uuid(record.user_id())?;
    let sheet_uuid = parse_sheet_uuid(record.sheet_id())?;

    Ok(RecordActiveModel {
        id: ActiveValue::Set(record_id),
        user_id: ActiveValue::Set(user_uuid),
        sheet_id: ActiveValue::Set(sheet_uuid),
        score: ActiveValue::Set(convert_score(*record.score())?),
        clear_type: ActiveValue::Set(DbClearType::from(*record.clear_type())),
        play_count: ActiveValue::Set(convert_play_count(*record.play_count())?),
        updated_at: ActiveValue::NotSet,
    })
}

pub fn active_model_for_update(
    record: &Record,
) -> Result<RecordActiveModel, RecordRepositoryError> {
    let record_id = parse_record_uuid(record.id())?;
    let user_uuid = parse_user_uuid(record.user_id())?;
    let sheet_uuid = parse_sheet_uuid(record.sheet_id())?;

    Ok(RecordActiveModel {
        id: ActiveValue::Unchanged(record_id),
        user_id: ActiveValue::Unchanged(user_uuid),
        sheet_id: ActiveValue::Unchanged(sheet_uuid),
        score: ActiveValue::Set(convert_score(*record.score())?),
        clear_type: ActiveValue::Set(DbClearType::from(*record.clear_type())),
        play_count: ActiveValue::Set(convert_play_count(*record.play_count())?),
        updated_at: ActiveValue::NotSet,
    })
}

pub fn convert_insert_error(
    err: sea_orm::DbErr,
    user_id: &str,
    sheet_id: &str,
) -> RecordRepositoryError {
    let message = err.to_string();
    if message.contains("fk_records_user") {
        return RecordRepositoryError::UserNotFound(user_id.to_owned());
    }

    if message.contains("fk_records_sheet") {
        return RecordRepositoryError::SheetNotFound(sheet_id.to_owned());
    }

    RecordRepositoryError::InternalError(AnyError::from(err))
}

pub fn convert_update_error(err: sea_orm::DbErr, record_id: &str) -> RecordRepositoryError {
    if matches!(err, sea_orm::DbErr::RecordNotUpdated) {
        tracing::debug!(record_id = %record_id, "Record not updated; underlying row may be missing");
    }

    RecordRepositoryError::InternalError(AnyError::from(err))
}

pub fn parse_user_uuid(user_id: &str) -> Result<Uuid, RecordRepositoryError> {
    Uuid::parse_str(user_id).map_err(|err| {
        tracing::debug!(error = %err, "Failed to parse user id");
        RecordRepositoryError::UserNotFound(user_id.to_owned())
    })
}

pub fn parse_sheet_uuid(sheet_id: &str) -> Result<Uuid, RecordRepositoryError> {
    Uuid::parse_str(sheet_id).map_err(|err| {
        tracing::debug!(error = %err, "Failed to parse sheet id");
        RecordRepositoryError::SheetNotFound(sheet_id.to_owned())
    })
}

fn parse_record_uuid(record_id: &str) -> Result<Uuid, RecordRepositoryError> {
    Uuid::parse_str(record_id).map_err(|err| {
        tracing::error!(error = %err, "Failed to parse record id");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })
}

fn convert_score(score: u32) -> Result<i32, RecordRepositoryError> {
    i32::try_from(score).map_err(|err| {
        tracing::error!(error = %err, "Score exceeds database range");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })
}

fn convert_play_count(play_count: u32) -> Result<i32, RecordRepositoryError> {
    i32::try_from(play_count).map_err(|err| {
        tracing::error!(error = %err, "Play count exceeds database range");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })
}

fn generate_record_uuid(user_id: &str, sheet_id: &str) -> Uuid {
    let mut upper = DefaultHasher::new();
    user_id.hash(&mut upper);
    sheet_id.hash(&mut upper);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    timestamp.hash(&mut upper);
    let high = upper.finish();

    let mut lower = DefaultHasher::new();
    sheet_id.hash(&mut lower);
    timestamp.wrapping_mul(31).hash(&mut lower);
    let low = lower.finish();

    let combined = ((high as u128) << 64) | (low as u128);
    Uuid::from_u128(combined)
}
