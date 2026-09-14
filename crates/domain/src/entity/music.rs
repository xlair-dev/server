use chrono::{DateTime, Utc};
use getset::{Getters, Setters};

use super::{asset::Asset, genre::Genre};

#[derive(Debug, Getters, Setters)]
pub struct Music {
    #[getset(get = "pub")]
    id: String,
    #[getset(get = "pub")]
    title: String,
    #[getset(get = "pub")]
    artist: String,
    #[getset(get = "pub")]
    bpm: f32,
    #[getset(get = "pub")]
    genre: Genre,
    #[getset(get = "pub")]
    jacket: Option<Asset>,
    #[getset(get = "pub")]
    audio: Option<Asset>,
    #[getset(get = "pub")]
    registration_date: DateTime<Utc>,
    #[getset(get = "pub")]
    is_test: bool,
}

#[allow(clippy::too_many_arguments)]
impl Music {
    pub fn new(
        id: String,
        title: String,
        artist: String,
        bpm: f32,
        genre: Genre,
        jacket: Option<Asset>,
        audio: Option<Asset>,
        registration_date: DateTime<Utc>,
        is_test: bool,
    ) -> Self {
        Self {
            id,
            title,
            artist,
            bpm,
            genre,
            jacket,
            audio,
            registration_date,
            is_test,
        }
    }
}
