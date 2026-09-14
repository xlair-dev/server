use domain::entity::difficulty::Difficulty;
use serde::Serialize;
use usecase::model::music::{AssetDto, MusicDto, MusicWithSheetsDto, SheetDto};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
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
pub struct AssetResponse {
    pub url: String,
    pub updated_at: String,
}

fn asset_response(
    asset: Option<AssetDto>,
    url: impl FnOnce(&str) -> String,
) -> Option<AssetResponse> {
    asset.map(|asset| AssetResponse {
        url: url(&asset.key),
        updated_at: asset.updated_at.to_rfc3339(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicResponse {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub bpm: f32,
    pub genre: String,
    pub jacket: Option<AssetResponse>,
    pub audio: Option<AssetResponse>,
    pub registration_date: String,
    pub is_test: bool,
}

impl From<MusicDto> for MusicResponse {
    fn from(value: MusicDto) -> Self {
        let id = value.id.clone();
        Self {
            id: value.id,
            title: value.title,
            artist: value.artist,
            bpm: value.bpm,
            genre: value.genre.to_string(),
            jacket: asset_response(value.jacket, |key| {
                format!("/musics/{id}/jacket/{}", asset_name(key))
            }),
            audio: asset_response(value.audio, |key| {
                format!("/musics/{id}/audio/{}", asset_name(key))
            }),
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
    pub chart: Option<AssetResponse>,
}

impl From<SheetDto> for SheetResponse {
    fn from(value: SheetDto) -> Self {
        let id = value.id.clone();
        Self {
            id: value.id,
            music_id: value.music_id,
            difficulty: difficulty_to_string(value.difficulty).to_owned(),
            level: value.level_value,
            notes_designer: value.notes_designer,
            chart: asset_response(value.chart, |key| {
                format!("/sheets/{id}/chart/{}", asset_name(key))
            }),
        }
    }
}

fn asset_name(key: &str) -> &str {
    key.rsplit('/').next().unwrap_or(key)
}

fn difficulty_to_string(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Basic => "basic",
        Difficulty::Advanced => "advanced",
        Difficulty::Master => "master",
    }
}
