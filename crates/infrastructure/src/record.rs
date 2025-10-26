use anyhow::Error as AnyError;
use domain::{
    entity::{level::Level, record::Record},
    repository::record::{RecordRepository, RecordRepositoryError, RecordWithMetadata},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter,
    prelude::Uuid,
};
use std::{
    collections::{HashMap, HashSet, hash_map::DefaultHasher},
    convert::TryFrom,
    hash::{Hash, Hasher},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::{debug, error, info, instrument};

use crate::{
    entities,
    entities::{
        prelude::Records, records::ActiveModel as RecordActiveModel,
        sea_orm_active_enums::ClearType as DbClearType,
    },
};

pub struct RecordRepositoryImpl {
    db: Arc<DbConn>,
}

impl RecordRepositoryImpl {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }

    async fn resolve_user_uuid(&self, user_id: &str) -> Result<Uuid, RecordRepositoryError> {
        let uuid = Uuid::parse_str(user_id).map_err(|err| {
            debug!(error = %err, "Failed to parse user id");
            RecordRepositoryError::UserNotFound(user_id.to_owned())
        })?;

        let user_exists = entities::users::Entity::find_by_id(uuid)
            .one(self.db.as_ref())
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
}

impl RecordRepository for RecordRepositoryImpl {
    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Record>, RecordRepositoryError> {
        debug!("Fetching records via SeaORM");

        let uuid = self.resolve_user_uuid(user_id).await?;

        let models = Records::find()
            .filter(entities::records::Column::UserId.eq(uuid))
            .all(self.db.as_ref())
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to fetch records");
                RecordRepositoryError::InternalError(AnyError::from(err))
            })?;

        info!(count = models.len(), "Records fetched successfully");
        Ok(models.into_iter().map(Record::from).collect())
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_with_metadata_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<Vec<RecordWithMetadata>, RecordRepositoryError> {
        debug!("Fetching records with metadata via SeaORM");

        let uuid = self.resolve_user_uuid(user_id).await?;

        let records_and_sheets = Records::find()
            .filter(entities::records::Column::UserId.eq(uuid))
            .find_also_related(entities::sheets::Entity)
            .all(self.db.as_ref())
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
            crate::entities::musics::Entity::find()
                .filter(
                    crate::entities::musics::Column::Id
                        .is_in(music_ids.iter().copied().collect::<Vec<_>>()),
                )
                .all(self.db.as_ref())
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
            let record = Record::from(record_model);
            result.push(RecordWithMetadata::new(record, level, is_test));
        }

        info!(
            count = result.len(),
            "Records with metadata fetched successfully"
        );
        Ok(result)
    }

    #[instrument(skip(self, record), fields(user_id = %record.user_id(), sheet_id = %record.sheet_id()))]
    async fn insert(&self, record: Record) -> Result<Record, RecordRepositoryError> {
        debug!("Persisting record via SeaORM");

        let user_id = record.user_id().to_owned();
        let sheet_id = record.sheet_id().to_owned();
        let active = to_active_model_for_insert(&record)?;

        match active.insert(self.db.as_ref()).await {
            Ok(model) => {
                info!(record_id = %model.id, "Record inserted successfully");
                Ok(model.into())
            }
            Err(err) => {
                error!(error = %err, "Failed to insert record");
                Err(convert_record_insert_error(err, &user_id, &sheet_id))
            }
        }
    }

    #[instrument(skip(self, record), fields(record_id = %record.id()))]
    async fn update(&self, record: Record) -> Result<Record, RecordRepositoryError> {
        debug!("Updating record via SeaORM");

        let record_id = record.id().to_owned();
        let active = to_active_model_for_update(&record)?;

        match active.update(self.db.as_ref()).await {
            Ok(model) => {
                info!(record_id = %model.id, "Record updated successfully");
                Ok(model.into())
            }
            Err(err) => {
                error!(error = %err, "Failed to update record");
                Err(convert_record_update_error(err, &record_id))
            }
        }
    }
}

fn to_active_model_for_insert(record: &Record) -> Result<RecordActiveModel, RecordRepositoryError> {
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

fn to_active_model_for_update(record: &Record) -> Result<RecordActiveModel, RecordRepositoryError> {
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

fn parse_user_uuid(user_id: &str) -> Result<Uuid, RecordRepositoryError> {
    Uuid::parse_str(user_id).map_err(|err| {
        debug!(error = %err, "Failed to parse user id");
        RecordRepositoryError::UserNotFound(user_id.to_owned())
    })
}

fn parse_sheet_uuid(sheet_id: &str) -> Result<Uuid, RecordRepositoryError> {
    Uuid::parse_str(sheet_id).map_err(|err| {
        debug!(error = %err, "Failed to parse sheet id");
        RecordRepositoryError::SheetNotFound(sheet_id.to_owned())
    })
}

fn parse_record_uuid(record_id: &str) -> Result<Uuid, RecordRepositoryError> {
    Uuid::parse_str(record_id).map_err(|err| {
        error!(error = %err, "Failed to parse record id");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })
}

fn convert_score(score: u32) -> Result<i32, RecordRepositoryError> {
    i32::try_from(score).map_err(|err| {
        error!(error = %err, "Score exceeds database range");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })
}

fn convert_play_count(play_count: u32) -> Result<i32, RecordRepositoryError> {
    i32::try_from(play_count).map_err(|err| {
        error!(error = %err, "Play count exceeds database range");
        RecordRepositoryError::InternalError(AnyError::from(err))
    })
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

fn convert_record_insert_error(err: DbErr, user_id: &str, sheet_id: &str) -> RecordRepositoryError {
    let message = err.to_string();
    if message.contains("fk_records_user") {
        return RecordRepositoryError::UserNotFound(user_id.to_owned());
    }

    if message.contains("fk_records_sheet") {
        return RecordRepositoryError::SheetNotFound(sheet_id.to_owned());
    }

    RecordRepositoryError::InternalError(AnyError::from(err))
}

fn convert_record_update_error(err: DbErr, record_id: &str) -> RecordRepositoryError {
    if matches!(err, DbErr::RecordNotUpdated) {
        debug!(record_id = %record_id, "Record not updated; underlying row may be missing");
    }

    RecordRepositoryError::InternalError(AnyError::from(err))
}
