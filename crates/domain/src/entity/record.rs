use chrono::NaiveDateTime;
use getset::{Getters, Setters};

use super::clear_type::ClearType;

#[derive(Debug, Getters, Setters)]
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
    updated_at: NaiveDateTime,
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
        updated_at: NaiveDateTime,
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
}
