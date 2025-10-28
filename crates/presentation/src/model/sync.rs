use domain::entity::difficulty::Difficulty;
use serde::Serialize;
use usecase::model::music::{MusicDto, MusicWithSheetsDto, SheetDto};

#[derive(Serialize)]
pub struct SyncItemResponse {
    pub music: MusicResponse,
    pub sheets: Vec<SheetResponse>,
}

impl From<MusicWithSheetsDto> for SyncItemResponse {
    fn from(value: MusicWithSheetsDto) -> Self {
        let music = MusicResponse::from(value.music);
        let sheets = value.sheets.into_iter().map(SheetResponse::from).collect();
        Self { music, sheets }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicResponse {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub bpm: f32,
    pub genre: String,
    pub jacket: String,
    pub registration_date: String,
    pub is_test: bool,
}

impl From<MusicDto> for MusicResponse {
    fn from(value: MusicDto) -> Self {
        Self {
            id: value.id,
            title: value.title,
            artist: value.artist,
            bpm: value.bpm,
            genre: value.genre.to_string(),
            jacket: value.jacket,
            registration_date: value.registration_date.to_rfc3339(),
            is_test: value.is_test,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetResponse {
    pub id: String,
    pub music_id: String,
    pub difficulty: String,
    pub level: f64,
    pub notes_designer: String,
}

impl From<SheetDto> for SheetResponse {
    fn from(value: SheetDto) -> Self {
        Self {
            id: value.id,
            music_id: value.music_id,
            difficulty: difficulty_to_string(value.difficulty).to_owned(),
            level: value.level_value,
            notes_designer: value.notes_designer,
        }
    }
}

fn difficulty_to_string(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Easy => "easy",
        Difficulty::Normal => "normal",
        Difficulty::Hard => "hard",
    }
}
