use std::collections::{HashMap, HashSet};

use anyhow::Error as AnyError;
use domain::{
    entity::level::Level,
    repository::record::{RecordRepositoryError, RecordWithMetadata},
};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, prelude::Uuid};
use tracing::{debug, error};

use crate::entities;

pub async fn ensure_user_exists(db: &DbConn, user_id: &str) -> Result<Uuid, RecordRepositoryError> {
    let uuid = crate::record::mapper::parse_user_uuid(user_id)?;

    let user_exists = entities::users::Entity::find_by_id(uuid)
        .one(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to verify user existence");
            RecordRepositoryError::InternalError(AnyError::from(err))
        })?;

    if user_exists.is_none() {
        debug!("User not found while querying records");
        return Err(RecordRepositoryError::UserNotFound(user_id.to_owned()));
    }

    Ok(uuid)
}

/// Loads records alongside sheet/music metadata. Relies on the `fk_records_sheet` and
/// `fk_sheets_music` constraints to guarantee referential integrity across tables.
pub async fn records_with_metadata(
    db: &DbConn,
    user_uuid: Uuid,
) -> Result<Vec<RecordWithMetadata>, RecordRepositoryError> {
    let records_and_sheets = entities::records::Entity::find()
        .filter(entities::records::Column::UserId.eq(user_uuid))
        .find_also_related(entities::sheets::Entity)
        .all(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to fetch records with metadata");
            RecordRepositoryError::InternalError(AnyError::from(err))
        })?;

    let mut music_ids = HashSet::new();
    for (record_model, sheet_model) in &records_and_sheets {
        let sheet = sheet_model.as_ref().ok_or_else(|| {
            error!(sheet_id = %record_model.sheet_id, "Sheet missing while loading records");
            RecordRepositoryError::SheetNotFound(record_model.sheet_id.to_string())
        })?;
        music_ids.insert(sheet.music_id);
    }

    let music_models = if music_ids.is_empty() {
        Vec::new()
    } else {
        entities::musics::Entity::find()
            .filter(
                entities::musics::Column::Id.is_in(music_ids.iter().copied().collect::<Vec<_>>()),
            )
            .all(db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to fetch music metadata");
                RecordRepositoryError::InternalError(AnyError::from(err))
            })?
    };

    let mut music_map = HashMap::with_capacity(music_models.len());
    for music in music_models {
        music_map.insert(music.id, music.is_test);
    }

    let mut result = Vec::with_capacity(records_and_sheets.len());
    for (record_model, sheet_model) in records_and_sheets {
        let sheet = sheet_model.ok_or_else(|| {
            error!(sheet_id = %record_model.sheet_id, "Sheet missing while composing metadata");
            RecordRepositoryError::SheetNotFound(record_model.sheet_id.to_string())
        })?;

        let is_test = music_map.get(&sheet.music_id).copied().ok_or_else(|| {
            error!(music_id = %sheet.music_id, "Music metadata missing for sheet");
            RecordRepositoryError::SheetNotFound(sheet.id.to_string())
        })?;

        let level = convert_level(sheet.level)?;
        let record = domain::entity::record::Record::from(record_model);
        result.push(RecordWithMetadata::new(record, level, is_test));
    }

    Ok(result)
}

fn convert_level(raw_level: i32) -> Result<Level, RecordRepositoryError> {
    if raw_level < 0 {
        error!(value = raw_level, "Level must be non-negative");
        return Err(RecordRepositoryError::InternalError(AnyError::msg(
            "negative level encountered",
        )));
    }

    let integer = u32::try_from(raw_level / 10).map_err(|err| {
        error!(error = %err, value = raw_level, "Failed to convert level integer part");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })?;
    let decimal = u32::try_from(raw_level % 10).map_err(|err| {
        error!(error = %err, value = raw_level, "Failed to convert level decimal part");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })?;

    Level::new(integer, decimal).map_err(|err| {
        error!(error = ?err, value = raw_level, "Invalid level value returned from database");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })
}
