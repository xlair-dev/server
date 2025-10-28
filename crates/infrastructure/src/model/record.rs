use std::convert::TryFrom;

use anyhow::Error as AnyError;
use domain::{
    entity::{clear_type::ClearType as DomainClearType, record::Record},
    repository::record::RecordRepositoryError,
};

use crate::entities::{
    records::Model as RecordModel, sea_orm_active_enums::ClearType as DbClearType,
};

/// Converts database record model to domain entity.
///
/// # Errors
/// Returns `InternalError` if the database contains invalid data (negative score or play_count).
/// This conversion assumes database integrity constraints ensure valid data.
impl TryFrom<RecordModel> for Record {
    type Error = RecordRepositoryError;

    fn try_from(model: RecordModel) -> Result<Self, Self::Error> {
        let id = model.id.to_string();
        let user_id = model.user_id.to_string();
        let sheet_id = model.sheet_id.to_string();
        let score = u32::try_from(model.score).map_err(|err| {
            tracing::warn!(error = %err, value = model.score, "Score from database must be non-negative");
            RecordRepositoryError::InternalError(AnyError::from(err))
        })?;
        let play_count = u32::try_from(model.play_count).map_err(|err| {
            tracing::warn!(error = %err, value = model.play_count, "Play count from database must be non-negative");
            RecordRepositoryError::InternalError(AnyError::from(err))
        })?;
        let updated_at = model.updated_at.with_timezone(&chrono::Utc);

        Ok(Record::new(
            id,
            user_id,
            sheet_id,
            score,
            model.clear_type.into(),
            play_count,
            updated_at,
        ))
    }
}

impl From<DbClearType> for DomainClearType {
    fn from(value: DbClearType) -> Self {
        match value {
            DbClearType::Failed => DomainClearType::Fail,
            DbClearType::Clear => DomainClearType::Clear,
            DbClearType::FullCombo => DomainClearType::FullCombo,
            DbClearType::AllPerfect => DomainClearType::AllPerfect,
        }
    }
}

impl From<DomainClearType> for DbClearType {
    fn from(value: DomainClearType) -> Self {
        match value {
            DomainClearType::Fail => DbClearType::Failed,
            DomainClearType::Clear => DbClearType::Clear,
            DomainClearType::FullCombo => DbClearType::FullCombo,
            DomainClearType::AllPerfect => DbClearType::AllPerfect,
        }
    }
}
