use chrono::{DateTime, Utc};
use domain::entity::{difficulty::Difficulty, genre::Genre, music::Music, sheet::Sheet};

#[derive(Debug)]
pub struct MusicDataInput {
    pub title: String,
    pub artist: String,
    pub bpm: f32,
    pub genre: Genre,
    pub jacket_key: Option<String>,
    pub music_key: Option<String>,
    pub registration_date: DateTime<Utc>,
    pub is_test: bool,
}

#[derive(Debug)]
pub struct SheetDataInput {
    pub difficulty: Difficulty,
    pub level: f64,
    pub notes_designer: String,
}

#[derive(Debug)]
pub struct SheetInput {
    pub id: String,
    pub difficulty: Difficulty,
    pub level: f64,
    pub notes_designer: String,
}

#[derive(Debug)]
pub struct CreateMusicInput {
    pub music: MusicDataInput,
    pub sheets: Vec<SheetDataInput>,
}

#[derive(Debug)]
pub struct UpdateMusicInput {
    pub music: MusicDataInput,
    pub sheets: Vec<SheetInput>,
}

#[derive(Debug)]
pub struct MusicDto {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub bpm: f32,
    pub genre: Genre,
    pub jacket_key: Option<String>,
    pub jacket_updated_at: Option<DateTime<Utc>>,
    pub music_key: Option<String>,
    pub music_updated_at: Option<DateTime<Utc>>,
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
        jacket_key: Option<String>,
        jacket_updated_at: Option<DateTime<Utc>>,
        music_key: Option<String>,
        music_updated_at: Option<DateTime<Utc>>,
        registration_date: DateTime<Utc>,
        is_test: bool,
    ) -> Self {
        Self {
            id,
            title,
            artist,
            bpm,
            genre,
            jacket_key,
            jacket_updated_at,
            music_key,
            music_updated_at,
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
            value.jacket_key().clone(),
            value.jacket_updated_at().to_owned(),
            value.music_key().clone(),
            value.music_updated_at().to_owned(),
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
    pub chart_key: Option<String>,
    pub chart_updated_at: Option<DateTime<Utc>>,
}

impl SheetDto {
    pub fn new(
        id: String,
        music_id: String,
        difficulty: Difficulty,
        level_value: f64,
        notes_designer: String,
        chart_key: Option<String>,
        chart_updated_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            music_id,
            difficulty,
            level_value,
            notes_designer,
            chart_key,
            chart_updated_at,
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
            value.chart_key().clone(),
            value.chart_updated_at().to_owned(),
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
