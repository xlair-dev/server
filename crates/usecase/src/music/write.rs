use std::collections::HashSet;

use domain::{
    entity::{difficulty::Difficulty, genre::Genre, level::Level, music::Music, sheet::Sheet},
    repository::{
        Repositories,
        music::{MusicRepository, MusicWithSheets},
    },
};

use super::{MusicUsecase, MusicUsecaseError};
use crate::{
    jacket::{JacketStorage, JacketUpload},
    model::music::{
        CreateMusicInput, MusicDataInput, MusicWithSheetsDto, SheetDataInput, SheetInput,
        UpdateMusicInput,
    },
};

impl<R: Repositories> MusicUsecase<R> {
    pub async fn upload_jacket(
        &self,
        storage: &dyn JacketStorage,
        music_id: String,
        jacket: JacketUpload,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        validate_music_id(&music_id)?;
        self.repositories
            .music()
            .find_with_sheets(&music_id)
            .await?;
        let jacket_url = storage
            .upload(&music_id, jacket)
            .await
            .map_err(MusicUsecaseError::JacketStorage)?;
        self.repositories
            .music()
            .update_jacket(&music_id, Some(jacket_url))
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    pub async fn delete_jacket(
        &self,
        storage: &dyn JacketStorage,
        music_id: String,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        validate_music_id(&music_id)?;
        let music = self
            .repositories
            .music()
            .find_with_sheets(&music_id)
            .await?;
        if music.music.jacket_image_url().is_some() {
            storage
                .delete(&music_id)
                .await
                .map_err(MusicUsecaseError::JacketStorage)?;
            return self
                .repositories
                .music()
                .update_jacket(&music_id, None)
                .await
                .map(Into::into)
                .map_err(Into::into);
        }
        Ok(music.into())
    }

    pub async fn create(
        &self,
        input: CreateMusicInput,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        let music_id = uuid::Uuid::new_v4().to_string();
        let music = build_music(
            input.music,
            music_id,
            input.sheets.into_iter().map(Into::into).collect(),
            None,
        )?;
        let created = self.repositories.music().insert_with_sheets(music).await?;
        Ok(created.into())
    }

    pub async fn update(
        &self,
        music_id: String,
        input: UpdateMusicInput,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        if uuid::Uuid::parse_str(&music_id).is_err() {
            return Err(MusicUsecaseError::InvalidInput(
                "music id is invalid".to_owned(),
            ));
        }
        let existing = self
            .repositories
            .music()
            .find_with_sheets(&music_id)
            .await?;
        let existing_sheet_ids: HashSet<&str> = existing
            .sheets
            .iter()
            .map(|sheet| sheet.id().as_str())
            .collect();
        let requested_sheet_ids: HashSet<&str> =
            input.sheets.iter().map(|sheet| sheet.id.as_str()).collect();
        if existing_sheet_ids != requested_sheet_ids {
            return Err(MusicUsecaseError::InvalidInput(
                "sheet ids must match the existing sheets".to_owned(),
            ));
        }
        let music = build_music(
            input.music,
            music_id,
            input.sheets.into_iter().map(Into::into).collect(),
            Some(existing),
        )?;
        let updated = self.repositories.music().update_with_sheets(music).await?;
        Ok(updated.into())
    }
}

fn validate_music_id(music_id: &str) -> Result<(), MusicUsecaseError> {
    if uuid::Uuid::parse_str(music_id).is_err() {
        return Err(MusicUsecaseError::InvalidInput(
            "music id is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn build_music(
    input: MusicDataInput,
    music_id: String,
    sheets_input: Vec<SheetBuildInput>,
    existing: Option<MusicWithSheets>,
) -> Result<MusicWithSheets, MusicUsecaseError> {
    let jacket = input.jacket.or_else(|| {
        existing
            .as_ref()
            .and_then(|music| music.music.jacket_image_url().clone())
    });
    if input.title.trim().is_empty()
        || input.artist.trim().is_empty()
        || !input.bpm.is_finite()
        || input.bpm <= 0.0
    {
        return Err(MusicUsecaseError::InvalidInput(
            "title, artist, and bpm must be valid".to_owned(),
        ));
    }
    if !matches!(input.genre, Genre::ORIGINAL) {
        return Err(MusicUsecaseError::InvalidInput(
            "genre is invalid".to_owned(),
        ));
    }
    if sheets_input.len() != 3 {
        return Err(MusicUsecaseError::InvalidInput(
            "exactly one sheet for each difficulty is required".to_owned(),
        ));
    }

    let mut sheets = Vec::with_capacity(3);
    let mut seen = [false; 3];
    for sheet in sheets_input {
        let difficulty = match sheet.data.difficulty {
            Difficulty::Easy => {
                if seen[0] {
                    return invalid_sheet();
                }
                seen[0] = true;
                Difficulty::Easy
            }
            Difficulty::Normal => {
                if seen[1] {
                    return invalid_sheet();
                }
                seen[1] = true;
                Difficulty::Normal
            }
            Difficulty::Hard => {
                if seen[2] {
                    return invalid_sheet();
                }
                seen[2] = true;
                Difficulty::Hard
            }
        };
        let level = level_from_value(sheet.data.level)?;
        let id = match (&existing, sheet.id) {
            (None, None) => uuid::Uuid::new_v4().to_string(),
            (Some(_), Some(id)) if uuid::Uuid::parse_str(&id).is_ok() => id,
            _ => return invalid_sheet(),
        };
        sheets.push(Sheet::new(
            id,
            music_id.clone(),
            difficulty,
            level,
            non_empty(sheet.data.notes_designer, "notesDesigner")?,
        ));
    }
    if seen != [true; 3] {
        return invalid_sheet();
    }
    Ok(MusicWithSheets::new(
        Music::new(
            music_id,
            input.title,
            input.artist,
            input.bpm,
            input.genre,
            jacket,
            input.registration_date,
            input.is_test,
        ),
        sheets,
    ))
}

struct SheetBuildInput {
    id: Option<String>,
    data: SheetDataInput,
}

impl From<SheetDataInput> for SheetBuildInput {
    fn from(data: SheetDataInput) -> Self {
        Self { id: None, data }
    }
}

impl From<SheetInput> for SheetBuildInput {
    fn from(value: SheetInput) -> Self {
        Self {
            id: Some(value.id),
            data: SheetDataInput {
                difficulty: value.difficulty,
                level: value.level,
                notes_designer: value.notes_designer,
            },
        }
    }
}

fn invalid_sheet<T>() -> Result<T, MusicUsecaseError> {
    Err(MusicUsecaseError::InvalidInput(
        "sheets are invalid".to_owned(),
    ))
}

fn non_empty(value: String, field: &str) -> Result<String, MusicUsecaseError> {
    if value.trim().is_empty() {
        return Err(MusicUsecaseError::InvalidInput(format!(
            "{field} must not be empty"
        )));
    }
    Ok(value)
}

fn level_from_value(value: f64) -> Result<Level, MusicUsecaseError> {
    if !value.is_finite() || !(1.0..=99.9).contains(&value) {
        return Err(MusicUsecaseError::InvalidInput(
            "sheet level is invalid".to_owned(),
        ));
    }
    let scaled = (value * 10.0).round();
    if (scaled / 10.0 - value).abs() > f64::EPSILON {
        return Err(MusicUsecaseError::InvalidInput(
            "sheet level is invalid".to_owned(),
        ));
    }
    Level::new((scaled as u32) / 10, (scaled as u32) % 10)
        .map_err(|_| MusicUsecaseError::InvalidInput("sheet level is invalid".to_owned()))
}
