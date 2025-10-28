use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

use anyhow::Error as AnyError;
use domain::{
    entity::record::Record,
    repository::record::{RecordRepositoryError, RecordWithMetadata},
};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder, prelude::Uuid};
use tracing::{debug, error, warn};

use crate::entities::{self, prelude::Records};

pub async fn records_by_user(
    db: &DbConn,
    user_id: &str,
) -> Result<Vec<Record>, RecordRepositoryError> {
    debug!("Resolving user before loading records");
    let uuid = ensure_user_exists(db, user_id).await?;

    let models = Records::find()
        .filter(entities::records::Column::UserId.eq(uuid))
        .order_by_asc(entities::records::Column::UpdatedAt)
        .all(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to fetch records");
            RecordRepositoryError::InternalError(AnyError::from(err))
        })?;

    models.into_iter().map(Record::try_from).collect()
}

pub async fn records_by_user_and_sheet_ids(
    db: &DbConn,
    user_id: &str,
    sheet_ids: &[String],
) -> Result<Vec<Record>, RecordRepositoryError> {
    debug!("Resolving user before loading records by sheet IDs");
    let uuid = ensure_user_exists(db, user_id).await?;

    if sheet_ids.is_empty() {
        debug!("No sheet IDs provided");
        return Ok(Vec::new());
    }

    let mut sheet_uuids = Vec::with_capacity(sheet_ids.len());
    for sheet_id in sheet_ids {
        let parsed = crate::record::adapter::parse_sheet_uuid(sheet_id)?;
        sheet_uuids.push(parsed);
    }

    let models = Records::find()
        .filter(entities::records::Column::UserId.eq(uuid))
        .filter(entities::records::Column::SheetId.is_in(sheet_uuids))
        .all(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to fetch records by sheet IDs");
            RecordRepositoryError::InternalError(AnyError::from(err))
        })?;

    models.into_iter().map(Record::try_from).collect()
}

pub async fn records_with_metadata_by_user(
    db: &DbConn,
    user_id: &str,
) -> Result<Vec<RecordWithMetadata>, RecordRepositoryError> {
    let uuid = ensure_user_exists(db, user_id).await?;
    records_with_metadata(db, uuid).await
}

pub async fn ensure_user_exists(db: &DbConn, user_id: &str) -> Result<Uuid, RecordRepositoryError> {
    let uuid = crate::record::adapter::parse_user_uuid(user_id)?;

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
async fn records_with_metadata(
    db: &DbConn,
    user_uuid: Uuid,
) -> Result<Vec<RecordWithMetadata>, RecordRepositoryError> {
    let records_and_sheets = entities::records::Entity::find()
        .filter(entities::records::Column::UserId.eq(user_uuid))
        .find_also_related(entities::sheets::Entity)
        .order_by_asc(entities::records::Column::UpdatedAt)
        .all(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to fetch records with metadata");
            RecordRepositoryError::InternalError(AnyError::from(err))
        })?;

    let mut music_ids = HashSet::new();
    for (record_model, sheet_model) in &records_and_sheets {
        let sheet = sheet_model.as_ref().ok_or_else(|| {
            warn!(sheet_id = %record_model.sheet_id, "Sheet missing while loading records");
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
            .order_by_asc(entities::musics::Column::Id)
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
            warn!(sheet_id = %record_model.sheet_id, "Sheet missing while composing metadata");
            RecordRepositoryError::SheetNotFound(record_model.sheet_id.to_string())
        })?;

        let is_test = music_map.get(&sheet.music_id).copied().ok_or_else(|| {
            warn!(music_id = %sheet.music_id, "Music metadata missing for sheet");
            RecordRepositoryError::SheetNotFound(sheet.id.to_string())
        })?;

        let level = crate::record::adapter::convert_level(sheet.level)?;
        let record = Record::try_from(record_model)?;
        result.push(RecordWithMetadata::new(record, level, is_test));
    }

    Ok(result)
}
