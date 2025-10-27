use domain::entity::{clear_type::ClearType as DomainClearType, record::Record};

use crate::entities::{
    records::Model as RecordModel, sea_orm_active_enums::ClearType as DbClearType,
};

impl From<RecordModel> for Record {
    fn from(model: RecordModel) -> Self {
        let id = model.id.to_string();
        let user_id = model.user_id.to_string();
        let sheet_id = model.sheet_id.to_string();
        let score = u32::try_from(model.score).expect("score must be non-negative");
        let play_count = u32::try_from(model.play_count).expect("play_count must be non-negative");
        let updated_at = model.updated_at.with_timezone(&chrono::Utc);

        Record::new(
            id,
            user_id,
            sheet_id,
            score,
            model.clear_type.into(),
            play_count,
            updated_at,
        )
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
