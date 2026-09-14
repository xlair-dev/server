use chrono::{DateTime, Utc};
use getset::{Getters, Setters};

use super::{difficulty::Difficulty, level::Level};

#[derive(Debug, Getters, Setters)]
pub struct Sheet {
    #[getset(get = "pub")]
    id: String,
    #[getset(get = "pub")]
    music_id: String,
    #[getset(get = "pub")]
    difficulty: Difficulty,
    #[getset(get = "pub")]
    level: Level,
    #[getset(get = "pub")]
    notes_designer: String,
    #[getset(get = "pub")]
    chart_key: Option<String>,
    #[getset(get = "pub", set = "pub")]
    chart_updated_at: Option<DateTime<Utc>>,
}

impl Sheet {
    pub fn new(
        id: String,
        music_id: String,
        difficulty: Difficulty,
        level: Level,
        notes_designer: String,
        chart_key: Option<String>,
    ) -> Self {
        Self {
            id,
            music_id,
            difficulty,
            level,
            notes_designer,
            chart_key,
            chart_updated_at: None,
        }
    }
}
