use super::{difficulty::Difficulty, level::Level};
use getset::{Getters, Setters};

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
}

impl Sheet {
    pub fn new(id: String, music_id: String, difficulty: Difficulty, level: Level) -> Self {
        Self {
            id,
            music_id,
            difficulty,
            level,
        }
    }
}
