use std::convert::TryFrom;

use anyhow::{Error as AnyError, anyhow};
use chrono::Utc;
use domain::{
    entity::{difficulty::Difficulty, genre::Genre, level::Level, music::Music, sheet::Sheet},
    repository::music::MusicRepositoryError,
};
use sea_orm::prelude::Decimal;
use tracing::warn;

use crate::entities::{
    musics::Model as MusicModel, sea_orm_active_enums::Difficulty as DbDifficulty,
    sheets::Model as SheetModel,
};

pub fn convert_music(model: MusicModel) -> Result<Music, MusicRepositoryError> {
    let bpm = convert_bpm(model.bpm)?;
    let genre = convert_genre(model.genre)?;
    let registration_date = model.registration_date.with_timezone(&Utc);

    Ok(Music::new(
        model.id.to_string(),
        model.title,
        model.artist,
        bpm,
        genre,
        model.jacket,
        registration_date,
        model.is_test,
    ))
}

pub fn convert_sheets(models: Vec<SheetModel>) -> Result<Vec<Sheet>, MusicRepositoryError> {
    let mut sheets = Vec::with_capacity(models.len());
    for model in models {
        sheets.push(convert_sheet(model)?);
    }
    Ok(sheets)
}

fn convert_sheet(model: SheetModel) -> Result<Sheet, MusicRepositoryError> {
    let difficulty = convert_difficulty(model.difficulty);
    let level = convert_level(model.level)?;

    Ok(Sheet::new(
        model.id.to_string(),
        model.music_id.to_string(),
        difficulty,
        level,
        model.notes_designer,
    ))
}

fn convert_bpm(bpm: Decimal) -> Result<f32, MusicRepositoryError> {
    let bpm_str = bpm.to_string();
    bpm_str.parse::<f32>().map_err(|err| {
        warn!(error = %err, value = %bpm, "Failed to parse BPM from decimal");
        MusicRepositoryError::InternalError(AnyError::from(err))
    })
}

fn convert_genre(value: i32) -> Result<Genre, MusicRepositoryError> {
    match value {
        0 => Ok(Genre::ORIGINAL),
        other => {
            warn!(value = other, "Unknown genre value; defaulting to ORIGINAL");
            Ok(Genre::ORIGINAL)
        }
    }
}

fn convert_level(raw_level: i32) -> Result<Level, MusicRepositoryError> {
    if raw_level < 0 {
        warn!(value = raw_level, "Level must be non-negative");
        return Err(MusicRepositoryError::InternalError(anyhow!(
            "negative level encountered"
        )));
    }

    let integer = u32::try_from(raw_level / 10).map_err(|err| {
        warn!(error = %err, value = raw_level, "Failed to convert level integer part");
        MusicRepositoryError::InternalError(AnyError::from(err))
    })?;

    let decimal = u32::try_from(raw_level % 10).map_err(|err| {
        warn!(error = %err, value = raw_level, "Failed to convert level decimal part");
        MusicRepositoryError::InternalError(AnyError::from(err))
    })?;

    Level::new(integer, decimal).map_err(|err| {
        warn!(error = ?err, value = raw_level, "Invalid level value returned from database");
        MusicRepositoryError::InternalError(AnyError::from(err))
    })
}

fn convert_difficulty(value: DbDifficulty) -> Difficulty {
    match value {
        DbDifficulty::Easy => Difficulty::Easy,
        DbDifficulty::Normal => Difficulty::Normal,
        DbDifficulty::Hard => Difficulty::Hard,
    }
}
