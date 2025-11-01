use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use anyhow::Error as AnyError;
use bigdecimal::{Signed, ToPrimitive};
use domain::{
    entity::record::Record,
    repository::record::{
        RecordRepositoryError, RecordWithMetadata, SheetScoreRankingRow, TotalScoreRankingRow,
    },
};
use sea_orm::{
    ColumnTrait, DbConn, EntityTrait, FromQueryResult, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait,
    prelude::Uuid,
    sea_query::{Alias, Expr},
    sqlx::types::BigDecimal,
};
use tracing::{debug, error, info, warn};

use crate::entities::{self, prelude::Records};

#[derive(Debug, FromQueryResult)]
struct SheetScoreRow {
    #[sea_orm(column_name = "user_id")]
    user_id: Uuid,
    #[sea_orm(column_name = "display_name")]
    display_name: String,
    #[sea_orm(column_name = "score")]
    score: i32,
}

#[derive(Debug, FromQueryResult)]
struct TotalScoreRow {
    #[sea_orm(column_name = "user_id")]
    user_id: Uuid,
    #[sea_orm(column_name = "display_name")]
    display_name: String,
    #[sea_orm(column_name = "total_score")]
    total_score: BigDecimal,
}

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

pub async fn public_high_scores_by_sheet(
    db: &DbConn,
    sheet_id: &str,
    limit: u64,
) -> Result<Vec<SheetScoreRankingRow>, RecordRepositoryError> {
    let sheet_uuid = crate::record::adapter::parse_sheet_uuid(sheet_id)?;

    debug!(
        sheet_id = %sheet_uuid,
        limit,
        "Fetching public high scores for sheet via SeaORM"
    );
    let rows = entities::records::Entity::find()
        .select_only()
        .column_as(entities::records::Column::UserId, "user_id")
        .column_as(entities::users::Column::DisplayName, "display_name")
        .column_as(entities::records::Column::Score, "score")
        .join(
            JoinType::InnerJoin,
            entities::records::Relation::Users.def(),
        )
        .filter(entities::records::Column::SheetId.eq(sheet_uuid))
        .filter(entities::users::Column::IsPublic.eq(true))
        .order_by_desc(entities::records::Column::Score)
        .order_by_asc(entities::records::Column::UpdatedAt)
        .limit(limit)
        .into_model::<SheetScoreRow>()
        .all(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to fetch sheet ranking");
            RecordRepositoryError::InternalError(AnyError::from(err))
        })?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        if row.score < 0 {
            let err = AnyError::msg("Negative score encountered in ranking query");
            error!("Ranking query returned negative score");
            return Err(RecordRepositoryError::InternalError(err));
        }
        let score = u32::try_from(row.score).map_err(|err| {
            error!(error = %err, "Failed to convert score to u32");
            RecordRepositoryError::InternalError(AnyError::from(err))
        })?;
        result.push(SheetScoreRankingRow::new(
            row.user_id.to_string(),
            row.display_name,
            score,
        ));
    }

    info!(
        sheet_id = %sheet_uuid,
        count = result.len(),
        "Sheet ranking fetched successfully"
    );
    Ok(result)
}

pub async fn public_total_score_ranking(
    db: &DbConn,
    limit: u64,
) -> Result<Vec<TotalScoreRankingRow>, RecordRepositoryError> {
    debug!(limit, "Fetching public total score ranking via SeaORM");
    let rows = entities::records::Entity::find()
        .select_only()
        .column_as(entities::users::Column::Id, "user_id")
        .column_as(entities::users::Column::DisplayName, "display_name")
        .column_as(
            Expr::col(entities::records::Column::Score)
                .sum()
                .cast_as(Alias::new("numeric")),
            "total_score",
        )
        .join(
            JoinType::InnerJoin,
            entities::records::Relation::Users.def(),
        )
        .filter(entities::users::Column::IsPublic.eq(true))
        .group_by(entities::users::Column::Id)
        .group_by(entities::users::Column::DisplayName)
        .order_by_desc(Expr::col(Alias::new("total_score")))
        .order_by_asc(entities::users::Column::DisplayName)
        .limit(limit)
        .into_model::<TotalScoreRow>()
        .all(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to fetch total score ranking");
            RecordRepositoryError::InternalError(AnyError::from(err))
        })?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        if row.total_score.is_negative() {
            let err = AnyError::msg("Negative total score encountered in ranking query");
            error!("Ranking query returned negative total score");
            return Err(RecordRepositoryError::InternalError(err));
        }
        let total_score = row.total_score.to_u64().ok_or_else(|| {
            let err = AnyError::msg("Failed to convert total score to u64");
            error!("Total score conversion failed for ranking");
            err
        })?;
        result.push(TotalScoreRankingRow::new(
            row.user_id.to_string(),
            row.display_name,
            total_score,
        ));
    }

    info!(
        count = result.len(),
        "Total score ranking fetched successfully"
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bigdecimal::BigDecimal;
    use sea_orm::{DatabaseBackend, MockDatabase, prelude::Uuid, sea_query::Value};

    use super::*;

    fn decimal_row(label: &str, value: Option<i64>) -> BTreeMap<String, Value> {
        let mapped_value = value
            .map(|v| Value::BigDecimal(Some(Box::new(BigDecimal::from(v)))))
            .unwrap_or(Value::BigDecimal(None));
        BTreeMap::from([(label.to_owned(), mapped_value)])
    }

    fn sheet_row(user_id: Uuid, display_name: &str, score: i32) -> BTreeMap<String, Value> {
        BTreeMap::from([
            ("user_id".to_owned(), Value::Uuid(Some(Box::new(user_id)))),
            (
                "display_name".to_owned(),
                Value::String(Some(Box::new(display_name.to_owned()))),
            ),
            ("score".to_owned(), Value::Int(Some(score))),
        ])
    }

    fn total_score_row(
        user_id: Uuid,
        display_name: &str,
        total_score: i64,
    ) -> BTreeMap<String, Value> {
        BTreeMap::from([
            ("user_id".to_owned(), Value::Uuid(Some(Box::new(user_id)))),
            (
                "display_name".to_owned(),
                Value::String(Some(Box::new(display_name.to_owned()))),
            ),
            (
                "total_score".to_owned(),
                Value::BigDecimal(Some(Box::new(BigDecimal::from(total_score)))),
            ),
        ])
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

    #[tokio::test]
    async fn public_high_scores_by_sheet_converts_rows() {
        let sheet_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").expect("valid uuid");
        let user_id = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").expect("valid uuid");
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![sheet_row(user_id, "Alice", 987_654)]])
            .into_connection();

        let result = public_high_scores_by_sheet(&db, &sheet_id.to_string(), 20)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        let entry = &result[0];
        assert_eq!(entry.user_id, user_id.to_string());
        assert_eq!(entry.display_name, "Alice");
        assert_eq!(entry.score, 987_654);
    }

    #[tokio::test]
    async fn public_total_score_ranking_converts_rows() {
        let user_id = Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("valid uuid");
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![total_score_row(user_id, "Bob", 1_234_567)]])
            .into_connection();

        let result = public_total_score_ranking(&db, 20).await.unwrap();

        assert_eq!(result.len(), 1);
        let entry = &result[0];
        assert_eq!(entry.user_id, user_id.to_string());
        assert_eq!(entry.display_name, "Bob");
        assert_eq!(entry.total_score, 1_234_567);
    }
}
