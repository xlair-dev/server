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
