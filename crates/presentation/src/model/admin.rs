use chrono::{DateTime, Utc};
use domain::entity::{difficulty::Difficulty, genre::Genre};
use serde::{Deserialize, Serialize};
use usecase::model::music::{
    CreateMusicInput, MusicDataInput, SheetDataInput, SheetInput, UpdateMusicInput,
};

use crate::{error::AppError, model::sync::SyncItemResponse};

#[derive(Deserialize)]
pub struct MusicListQuery {
    pub cursor: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicListResponse {
    pub items: Vec<SyncItemResponse>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbSynchronizationResponse {
    pub updated_users: u64,
    pub updated_ratings: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicDataRequest {
    pub title: String,
    pub artist: String,
    pub bpm: f32,
    pub genre: String,
    pub jacket: String,
    pub registration_date: String,
    pub is_test: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMusicRequest {
    #[serde(flatten)]
    pub music: MusicDataRequest,
    pub sheets: Vec<SheetDataRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMusicRequest {
    #[serde(flatten)]
    pub music: MusicDataRequest,
    pub sheets: Vec<SheetRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetDataRequest {
    pub difficulty: String,
    pub level: f64,
    pub notes_designer: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetRequest {
    pub id: String,
    pub difficulty: String,
    pub level: f64,
    pub notes_designer: String,
}

impl TryFrom<MusicDataRequest> for MusicDataInput {
    type Error = AppError;

    fn try_from(request: MusicDataRequest) -> Result<Self, Self::Error> {
        let registration_date = DateTime::parse_from_rfc3339(&request.registration_date)
            .map_err(|_| AppError::bad_request("registrationDate is invalid"))?
            .with_timezone(&Utc);
        let genre = match request.genre.as_str() {
            "ORIGINAL" => Genre::ORIGINAL,
            _ => return Err(AppError::bad_request("genre is invalid")),
        };
        Ok(Self {
            title: request.title,
            artist: request.artist,
            bpm: request.bpm,
            genre,
            jacket: request.jacket,
            registration_date,
            is_test: request.is_test,
        })
    }
}

fn parse_difficulty(value: &str) -> Result<Difficulty, AppError> {
    match value {
        "easy" => Ok(Difficulty::Easy),
        "normal" => Ok(Difficulty::Normal),
        "hard" => Ok(Difficulty::Hard),
        _ => Err(AppError::bad_request("difficulty is invalid")),
    }
}

impl TryFrom<SheetDataRequest> for SheetDataInput {
    type Error = AppError;

    fn try_from(request: SheetDataRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            difficulty: parse_difficulty(&request.difficulty)?,
            level: request.level,
            notes_designer: request.notes_designer,
        })
    }
}

impl TryFrom<SheetRequest> for SheetInput {
    type Error = AppError;

    fn try_from(request: SheetRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            id: request.id,
            difficulty: parse_difficulty(&request.difficulty)?,
            level: request.level,
            notes_designer: request.notes_designer,
        })
    }
}

impl TryFrom<CreateMusicRequest> for CreateMusicInput {
    type Error = AppError;

    fn try_from(request: CreateMusicRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            music: request.music.try_into()?,
            sheets: request
                .sheets
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<UpdateMusicRequest> for UpdateMusicInput {
    type Error = AppError;

    fn try_from(request: UpdateMusicRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            music: request.music.try_into()?,
            sheets: request
                .sheets
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
