use std::convert::TryFrom;

use anyhow::{Error as AnyError, anyhow};
use chrono::Utc;
use domain::{
    entity::{
        asset::Asset, difficulty::Difficulty, genre::Genre, level::Level, music::Music,
        sheet::Sheet,
    },
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

    let jacket = asset(model.jacket_key, model.jacket_updated_at)?;
    let audio = asset(model.audio_key, model.audio_updated_at)?;
    Ok(Music::new(
        model.id.to_string(),
        model.title,
        model.artist,
        bpm,
        genre,
        jacket,
        audio,
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

    let chart = asset(model.chart_key, model.chart_updated_at)?;
    Ok(Sheet::new(
        model.id.to_string(),
        model.music_id.to_string(),
        difficulty,
        level,
        model.notes_designer,
        chart,
    ))
}

fn asset(
    key: Option<String>,
    updated_at: Option<sea_orm::prelude::DateTimeWithTimeZone>,
) -> Result<Option<Asset>, MusicRepositoryError> {
    match (key, updated_at) {
        (Some(key), Some(updated_at)) => Ok(Some(Asset::new(key, updated_at.with_timezone(&Utc)))),
        (None, None) => Ok(None),
        _ => Err(MusicRepositoryError::InternalError(AnyError::msg(
            "asset key and updated_at must be present together",
        ))),
    }
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
        1 => Ok(Genre::EXTERNAL),
        2 => Ok(Genre::OTHER),
        other => {
            warn!(value = other, "Unknown genre value returned from database");
            Err(MusicRepositoryError::InternalError(anyhow!(
                "unknown genre value: {other}"
            )))
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
        DbDifficulty::Basic => Difficulty::Basic,
        DbDifficulty::Advanced => Difficulty::Advanced,
        DbDifficulty::Master => Difficulty::Master,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{FixedOffset, TimeZone, Utc};

    use super::*;

    #[test]
    fn asset_requires_key_and_updated_at_together() {
        let updated_at = Utc
            .with_ymd_and_hms(2025, 10, 1, 12, 0, 0)
            .unwrap()
            .with_timezone(&FixedOffset::east_opt(0).unwrap());

        assert!(asset(Some("jacket.png".to_owned()), None).is_err());
        assert!(asset(None, Some(updated_at)).is_err());
    }

    #[test]
    fn asset_converts_database_values() {
        let updated_at = Utc
            .with_ymd_and_hms(2025, 10, 1, 12, 0, 0)
            .unwrap()
            .with_timezone(&FixedOffset::east_opt(9 * 60 * 60).unwrap());

        let converted = asset(Some("jacket.png".to_owned()), Some(updated_at))
            .unwrap()
            .unwrap();

        assert_eq!(converted.key(), "jacket.png");
        assert_eq!(
            converted.updated_at(),
            Utc.with_ymd_and_hms(2025, 10, 1, 12, 0, 0).unwrap()
        );
    }
}
