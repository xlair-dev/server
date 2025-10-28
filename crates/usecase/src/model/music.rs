use chrono::{DateTime, Utc};
use domain::entity::{difficulty::Difficulty, genre::Genre, music::Music, sheet::Sheet};

#[derive(Debug)]
pub struct MusicDto {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub bpm: f32,
    pub genre: Genre,
    pub jacket: String,
    pub registration_date: DateTime<Utc>,
    pub is_test: bool,
}

impl MusicDto {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        title: String,
        artist: String,
        bpm: f32,
        genre: Genre,
        jacket: String,
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
            registration_date,
            is_test,
        }
    }
}

impl From<Music> for MusicDto {
    fn from(value: Music) -> Self {
        Self::new(
            value.id().to_owned(),
            value.title().to_owned(),
            value.artist().to_owned(),
            *value.bpm(),
            *value.genre(),
            value.jacket_image_url().to_owned(),
            value.registration_date().to_owned(),
            *value.is_test(),
        )
    }
}

#[derive(Debug)]
pub struct SheetDto {
    pub id: String,
    pub music_id: String,
    pub difficulty: Difficulty,
    pub level_value: f64,
    pub notes_designer: String,
}

impl SheetDto {
    pub fn new(
        id: String,
        music_id: String,
        difficulty: Difficulty,
        level_value: f64,
        notes_designer: String,
    ) -> Self {
        Self {
            id,
            music_id,
            difficulty,
            level_value,
            notes_designer,
        }
    }
}

impl From<Sheet> for SheetDto {
    fn from(value: Sheet) -> Self {
        Self::new(
            value.id().to_owned(),
            value.music_id().to_owned(),
            *value.difficulty(),
            value.level().value(),
            value.notes_designer().to_owned(),
        )
    }
}

#[derive(Debug)]
pub struct MusicWithSheetsDto {
    pub music: MusicDto,
    pub sheets: Vec<SheetDto>,
}

impl MusicWithSheetsDto {
    pub fn new(music: MusicDto, sheets: Vec<SheetDto>) -> Self {
        Self { music, sheets }
    }
}
