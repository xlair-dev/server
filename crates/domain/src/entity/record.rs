use chrono::{DateTime, Utc};
use getset::{Getters, Setters};

use super::clear_type::ClearType;

#[derive(Debug, Clone, Getters, Setters)]
pub struct Record {
    #[getset(get = "pub")]
    id: String,
    #[getset(get = "pub")]
    user_id: String,
    #[getset(get = "pub")]
    sheet_id: String,
    #[getset(get = "pub", set = "pub")]
    score: u32,
    #[getset(get = "pub", set = "pub")]
    clear_type: ClearType,
    #[getset(get = "pub", set = "pub")]
    play_count: u32,
    #[getset(get = "pub", set = "pub")]
    updated_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
impl Record {
    pub fn new(
        id: String,
        user_id: String,
        sheet_id: String,
        score: u32,
        clear_type: ClearType,
        play_count: u32,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            sheet_id,
            score,
            clear_type,
            play_count,
            updated_at,
        }
    }

    pub fn new_from_submission(
        user_id: String,
        sheet_id: String,
        score: u32,
        clear_type: ClearType,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self::new(
            String::new(),
            user_id,
            sheet_id,
            score,
            clear_type,
            1,
            updated_at,
        )
    }

    pub fn apply_submission(
        &mut self,
        score: u32,
        clear_type: ClearType,
        updated_at: DateTime<Utc>,
    ) {
        self.set_play_count(self.play_count() + 1);
        if score > *self.score() {
            self.set_score(score);
        }
        if is_better_clear(clear_type, *self.clear_type()) {
            self.set_clear_type(clear_type);
        }
        self.set_updated_at(updated_at);
    }
}

fn is_better_clear(new: ClearType, current: ClearType) -> bool {
    clear_type_rank(new) > clear_type_rank(current)
}

fn clear_type_rank(clear_type: ClearType) -> u8 {
    match clear_type {
        ClearType::Fail => 0,
        ClearType::Clear => 1,
        ClearType::FullCombo => 2,
        ClearType::AllPerfect => 3,
    }
}
