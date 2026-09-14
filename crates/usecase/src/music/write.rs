use std::collections::HashSet;

use domain::{
    entity::{difficulty::Difficulty, level::Level, music::Music, sheet::Sheet},
    repository::{
        Repositories,
        music::{MusicRepository, MusicWithSheets},
    },
};
use tracing::warn;

use super::{MusicUsecase, MusicUsecaseError};
use crate::{
    asset::{AssetKind, AssetStorage, AssetUpload},
    model::music::{
        CreateMusicInput, MusicDataInput, MusicWithSheetsDto, SheetDataInput, SheetInput,
        UpdateMusicInput,
    },
};

impl<R: Repositories> MusicUsecase<R> {
    pub async fn upload_jacket(
        &self,
        storage: &dyn AssetStorage,
        music_id: String,
        jacket: AssetUpload,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        validate_music_id(&music_id)?;
        let existing = self
            .repositories
            .music()
            .find_with_sheets(&music_id)
            .await?;
        let previous_jacket_url = existing.music.jacket_key().clone();
        let jacket_url = storage
            .upload(AssetKind::Jacket, &music_id, jacket)
            .await
            .map_err(MusicUsecaseError::AssetStorage)?;
        let updated = match self
            .repositories
            .music()
            .update_jacket_key(&music_id, Some(jacket_url.clone()))
            .await
        {
            Ok(updated) => updated,
            Err(error) => {
                delete_uploaded(storage, &jacket_url).await;
                return Err(error.into());
            }
        };
        if previous_jacket_url.as_deref() != Some(jacket_url.as_str())
            && let Some(previous_jacket_url) = previous_jacket_url
            && let Err(error) = storage.delete(&previous_jacket_url).await
        {
            warn!(
                error = %error,
                jacket_url = %previous_jacket_url,
                "Failed to clean up previous jacket"
            );
        }
        Ok(updated.into())
    }

    pub async fn delete_jacket(
        &self,
        storage: &dyn AssetStorage,
        music_id: String,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        validate_music_id(&music_id)?;
        let music = self
            .repositories
            .music()
            .find_with_sheets(&music_id)
            .await?;
        if let Some(jacket_key) = music.music.jacket_key().clone() {
            let updated = self
                .repositories
                .music()
                .update_jacket_key(&music_id, None)
                .await
                .map_err(MusicUsecaseError::from)?;
            delete_existing(storage, &jacket_key).await;
            return Ok(updated.into());
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

    pub async fn upload_audio(
        &self,
        storage: &dyn AssetStorage,
        music_id: String,
        audio: AssetUpload,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        validate_music_id(&music_id)?;
        let existing = self
            .repositories
            .music()
            .find_with_sheets(&music_id)
            .await?;
        let previous_key = existing.music.music_key().clone();
        let key = storage
            .upload(AssetKind::Audio, &music_id, audio)
            .await
            .map_err(MusicUsecaseError::AssetStorage)?;
        let updated = match self
            .repositories
            .music()
            .update_music_key(&music_id, Some(key.clone()))
            .await
        {
            Ok(updated) => updated,
            Err(error) => {
                delete_uploaded(storage, &key).await;
                return Err(error.into());
            }
        };
        delete_previous(storage, previous_key, &key).await;
        Ok(updated.into())
    }

    pub async fn upload_chart(
        &self,
        storage: &dyn AssetStorage,
        sheet_id: String,
        chart: AssetUpload,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        let sheet = self.repositories.music().find_sheet(&sheet_id).await?;
        let previous_key = sheet.chart_key().clone();
        let key = storage
            .upload(AssetKind::Chart, &sheet_id, chart)
            .await
            .map_err(MusicUsecaseError::AssetStorage)?;
        let updated = match self
            .repositories
            .music()
            .update_chart_key(&sheet_id, Some(key.clone()))
            .await
        {
            Ok(updated) => updated,
            Err(error) => {
                delete_uploaded(storage, &key).await;
                return Err(error.into());
            }
        };
        delete_previous(storage, previous_key, &key).await;
        Ok(updated.into())
    }

    pub async fn delete_audio(
        &self,
        storage: &dyn AssetStorage,
        music_id: String,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        validate_music_id(&music_id)?;
        let music = self
            .repositories
            .music()
            .find_with_sheets(&music_id)
            .await?;
        if let Some(key) = music.music.music_key().clone() {
            let updated = self
                .repositories
                .music()
                .update_music_key(&music_id, None)
                .await
                .map_err(MusicUsecaseError::from)?;
            delete_existing(storage, &key).await;
            return Ok(updated.into());
        }
        Ok(music.into())
    }

    pub async fn delete_chart(
        &self,
        storage: &dyn AssetStorage,
        sheet_id: String,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        let sheet = self.repositories.music().find_sheet(&sheet_id).await?;
        if let Some(key) = sheet.chart_key().clone() {
            let updated = self
                .repositories
                .music()
                .update_chart_key(&sheet_id, None)
                .await
                .map_err(MusicUsecaseError::from)?;
            delete_existing(storage, &key).await;
            return Ok(updated.into());
        }
        Ok(self
            .repositories
            .music()
            .find_with_sheets(&sheet.music_id().to_owned())
            .await?
            .into())
    }
}

async fn delete_previous(storage: &dyn AssetStorage, previous_key: Option<String>, key: &str) {
    if let Some(previous_key) = previous_key
        && previous_key != key
        && let Err(error) = storage.delete(&previous_key).await
    {
        tracing::warn!(error = ?error, asset_key = %previous_key, "Failed to clean up previous asset");
    }
}

async fn delete_uploaded(storage: &dyn AssetStorage, key: &str) {
    if let Err(error) = storage.delete(key).await {
        warn!(error = %error, asset_key = %key, "Failed to clean up uploaded asset");
    }
}

async fn delete_existing(storage: &dyn AssetStorage, key: &str) {
    if let Err(error) = storage.delete(key).await {
        warn!(error = %error, asset_key = %key, "Failed to delete existing asset");
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
    let jacket = input.jacket_key.or_else(|| {
        existing
            .as_ref()
            .and_then(|music| music.music.jacket_key().clone())
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
    if sheets_input.len() != 3 {
        return Err(MusicUsecaseError::InvalidInput(
            "exactly one sheet for each difficulty is required".to_owned(),
        ));
    }

    let mut sheets = Vec::with_capacity(3);
    let mut seen = [false; 3];
    for sheet in sheets_input {
        let difficulty = match sheet.data.difficulty {
            Difficulty::Basic => {
                if seen[0] {
                    return invalid_sheet();
                }
                seen[0] = true;
                Difficulty::Basic
            }
            Difficulty::Advanced => {
                if seen[1] {
                    return invalid_sheet();
                }
                seen[1] = true;
                Difficulty::Advanced
            }
            Difficulty::Master => {
                if seen[2] {
                    return invalid_sheet();
                }
                seen[2] = true;
                Difficulty::Master
            }
        };
        let level = level_from_value(sheet.data.level)?;
        let id = match (&existing, sheet.id) {
            (None, None) => uuid::Uuid::new_v4().to_string(),
            (Some(_), Some(id)) if uuid::Uuid::parse_str(&id).is_ok() => id,
            _ => return invalid_sheet(),
        };
        let mut sheet = Sheet::new(
            id.clone(),
            music_id.clone(),
            difficulty,
            level,
            non_empty(sheet.data.notes_designer, "notesDesigner")?,
            existing.as_ref().and_then(|music| {
                music
                    .sheets
                    .iter()
                    .find(|existing_sheet| existing_sheet.id() == &id)
                    .and_then(|sheet| sheet.chart_key().clone())
            }),
        );
        if let Some(existing_sheet) = existing
            .as_ref()
            .and_then(|music| music.sheets.iter().find(|sheet| sheet.id() == &id))
        {
            sheet.set_chart_updated_at(*existing_sheet.chart_updated_at());
        }
        sheets.push(sheet);
    }
    if seen != [true; 3] {
        return invalid_sheet();
    }
    let mut music = Music::new(
        music_id,
        input.title,
        input.artist,
        input.bpm,
        input.genre,
        jacket,
        input.music_key.or_else(|| {
            existing
                .as_ref()
                .and_then(|music| music.music.music_key().clone())
        }),
        input.registration_date,
        input.is_test,
    );
    if let Some(existing_music) = existing.as_ref().map(|music| &music.music) {
        music.set_jacket_updated_at(*existing_music.jacket_updated_at());
        music.set_music_updated_at(*existing_music.music_updated_at());
    }
    Ok(MusicWithSheets::new(music, sheets))
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
