use anyhow::Error as AnyError;
use domain::{
    entity::{level::Level, record::Record},
    repository::record::RecordRepositoryError,
};
use sea_orm::{ActiveValue, DbErr, prelude::Uuid};

use crate::entities::{
    records::ActiveModel as RecordActiveModel, sea_orm_active_enums::ClearType as DbClearType,
};

/// Builds an `ActiveModel` for inserts, delegating `id` generation to the
/// database default (`gen_random_uuid()` defined in
/// `m20251007_000004_create_records_table`).
pub fn active_model_for_insert(
    record: &Record,
) -> Result<RecordActiveModel, RecordRepositoryError> {
    let record_id = if record.id().is_empty() {
        ActiveValue::NotSet
    } else {
        ActiveValue::Set(parse_record_uuid(record.id())?)
    };

    let user_uuid = parse_user_uuid(record.user_id())?;
    let sheet_uuid = parse_sheet_uuid(record.sheet_id())?;

    Ok(RecordActiveModel {
        id: record_id,
        user_id: ActiveValue::Set(user_uuid),
        sheet_id: ActiveValue::Set(sheet_uuid),
        score: ActiveValue::Set(convert_score(*record.score())?),
        clear_type: ActiveValue::Set(DbClearType::from(*record.clear_type())),
        play_count: ActiveValue::Set(convert_play_count(*record.play_count())?),
        updated_at: ActiveValue::Set((*record.updated_at()).into()),
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
        updated_at: ActiveValue::Set((*record.updated_at()).into()),
    })
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
        tracing::debug!(error = %err, "Failed to parse record id");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })
}

pub fn convert_level(raw_level: i32) -> Result<Level, RecordRepositoryError> {
    if raw_level < 0 {
        tracing::warn!(value = raw_level, "Level must be non-negative");
        return Err(RecordRepositoryError::InternalError(AnyError::msg(
            "negative level encountered",
        )));
    }

    let integer = u32::try_from(raw_level / 10).map_err(|err| {
        tracing::warn!(error = %err, value = raw_level, "Failed to convert level integer part");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })?;
    let decimal = u32::try_from(raw_level % 10).map_err(|err| {
        tracing::warn!(error = %err, value = raw_level, "Failed to convert level decimal part");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })?;

    Level::new(integer, decimal).map_err(|err| {
        tracing::warn!(error = ?err, value = raw_level, "Invalid level value returned from database");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })
}

fn convert_score(score: u32) -> Result<i32, RecordRepositoryError> {
    i32::try_from(score).map_err(|err| {
        tracing::warn!(error = %err, "Score exceeds database range");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })
}

fn convert_play_count(play_count: u32) -> Result<i32, RecordRepositoryError> {
    i32::try_from(play_count).map_err(|err| {
        tracing::warn!(error = %err, "Play count exceeds database range");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })
}

pub fn convert_insert_error(err: DbErr, user_id: &str, sheet_id: &str) -> RecordRepositoryError {
    let message = err.to_string();
    if message.contains("fk_records_user") {
        tracing::warn!(user_id = %user_id, "User not found for foreign key constraint");
        return RecordRepositoryError::UserNotFound(user_id.to_owned());
    }

    if message.contains("fk_records_sheet") {
        tracing::warn!(sheet_id = %sheet_id, "Sheet not found for foreign key constraint");
        return RecordRepositoryError::SheetNotFound(sheet_id.to_owned());
    }

    tracing::error!(error = %err, "Failed to insert record");
    RecordRepositoryError::InternalError(AnyError::from(err))
}

pub fn convert_update_error(err: DbErr, record_id: &str) -> RecordRepositoryError {
    if matches!(err, DbErr::RecordNotUpdated) {
        tracing::debug!(record_id = %record_id, "Record not updated; underlying row may be missing");
        RecordRepositoryError::InternalError(AnyError::from(err))
    } else {
        tracing::error!(error = %err, record_id = %record_id, "Failed to update record");
        RecordRepositoryError::InternalError(AnyError::from(err))
    }
}
