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
}

impl Sheet {
    pub fn new(
        id: String,
        music_id: String,
        difficulty: Difficulty,
        level: Level,
        notes_designer: String,
    ) -> Self {
        Self {
            id,
            music_id,
            difficulty,
            level,
            notes_designer,
        }
    }
}
