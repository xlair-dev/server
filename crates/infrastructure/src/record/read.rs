use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use anyhow::Error as AnyError;
use bigdecimal::{Signed, ToPrimitive};
use domain::{
    entity::record::Record,
    repository::record::{RecordRepositoryError, RecordWithMetadata},
};
use sea_orm::{
    ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    prelude::Uuid,
    sea_query::{Alias, Expr},
    sqlx::types::BigDecimal,
};
use tracing::{debug, error, info, warn};

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

/// Aggregates record scores across all users. Casts SUM(...) to NUMERIC to
/// stabilize Postgres' return type regardless of column width.
pub async fn sum_scores(db: &DbConn) -> Result<u64, RecordRepositoryError> {
    debug!("Summing record scores via SeaORM");
    let sum = entities::records::Entity::find()
        .select_only()
        .column_as(
            Expr::col(entities::records::Column::Score)
                .sum()
                .cast_as(Alias::new("numeric")),
            "sum",
        )
        .into_tuple::<Option<BigDecimal>>()
        .one(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to sum record scores");
            RecordRepositoryError::InternalError(AnyError::from(err))
        })?
        .flatten()
        .unwrap_or_else(|| BigDecimal::from(0u8));

    if sum.is_negative() {
        let err = AnyError::msg("Database returned negative record score sum");
        error!("Record score sum returned negative value");
        return Err(RecordRepositoryError::InternalError(err));
    }

    let sum = sum.to_u64().ok_or_else(|| {
        let err = AnyError::msg("Record score sum cannot fit in u64");
        error!("Record score sum overflowed u64 conversion");
        RecordRepositoryError::InternalError(err)
    })?;

    info!(total_score = sum, "Record scores summed successfully");
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use sea_orm::sea_query::Value;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use std::collections::BTreeMap;

    fn decimal_row(label: &str, value: Option<i64>) -> BTreeMap<String, Value> {
        let mapped_value = value
            .map(|v| Value::BigDecimal(Some(Box::new(BigDecimal::from(v)))))
            .unwrap_or(Value::BigDecimal(None));
        BTreeMap::from([(label.to_owned(), mapped_value)])
    }

    #[tokio::test]
    async fn sum_scores_handles_numeric_rows() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![decimal_row("sum", Some(1_234))]])
            .into_connection();

        let result = sum_scores(&db).await.unwrap();
        assert_eq!(result, 1_234);
    }

    #[tokio::test]
    async fn sum_scores_defaults_to_zero() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![decimal_row("sum", None)]])
            .into_connection();

        let result = sum_scores(&db).await.unwrap();
        assert_eq!(result, 0);
    }
}
