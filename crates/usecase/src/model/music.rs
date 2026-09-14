use chrono::{DateTime, Utc};
use domain::entity::{
    asset::Asset, difficulty::Difficulty, genre::Genre, music::Music, sheet::Sheet,
};

#[derive(Debug)]
pub struct MusicDataInput {
    pub title: String,
    pub artist: String,
    pub bpm: f32,
    pub genre: Genre,
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
    pub jacket: Option<AssetDto>,
    pub audio: Option<AssetDto>,
    pub registration_date: DateTime<Utc>,
    pub is_test: bool,
}

#[derive(Debug)]
pub struct AssetDto {
    pub key: String,
    pub updated_at: DateTime<Utc>,
}

impl From<Asset> for AssetDto {
    fn from(value: Asset) -> Self {
        Self {
            key: value.key().to_owned(),
            updated_at: value.updated_at(),
        }
    }
}

impl MusicDto {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        title: String,
        artist: String,
        bpm: f32,
        genre: Genre,
        jacket: Option<AssetDto>,
        audio: Option<AssetDto>,
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

impl From<Music> for MusicDto {
    fn from(value: Music) -> Self {
        Self::new(
            value.id().to_owned(),
            value.title().to_owned(),
            value.artist().to_owned(),
            *value.bpm(),
            *value.genre(),
            value.jacket().clone().map(Into::into),
            value.audio().clone().map(Into::into),
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
    pub chart: Option<AssetDto>,
}

impl SheetDto {
    pub fn new(
        id: String,
        music_id: String,
        difficulty: Difficulty,
        level_value: f64,
        notes_designer: String,
        chart: Option<AssetDto>,
    ) -> Self {
        Self {
            id,
            music_id,
            difficulty,
            level_value,
            notes_designer,
            chart,
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
            value.chart().clone().map(Into::into),
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
