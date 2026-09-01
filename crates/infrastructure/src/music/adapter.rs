use std::convert::TryFrom;

use anyhow::{Error as AnyError, anyhow};
use chrono::Utc;
use domain::{
    entity::{difficulty::Difficulty, genre::Genre, level::Level, music::Music, sheet::Sheet},
    repository::music::MusicRepositoryError,
};
use sea_orm::{
    ActiveValue,
    prelude::{Decimal, Uuid},
};
use tracing::warn;

use crate::entities::{
    musics::{ActiveModel as MusicActiveModel, Model as MusicModel},
    sea_orm_active_enums::Difficulty as DbDifficulty,
    sheets::{ActiveModel as SheetActiveModel, Model as SheetModel},
};

pub fn music_active_model_for_insert(
    music: &Music,
) -> Result<MusicActiveModel, MusicRepositoryError> {
    music_active_model(music, ActiveValue::Set(parse_uuid(music.id())?))
}

pub fn music_active_model_for_update(
    music: &Music,
) -> Result<MusicActiveModel, MusicRepositoryError> {
    music_active_model(music, ActiveValue::Unchanged(parse_uuid(music.id())?))
}

fn music_active_model(
    music: &Music,
    id: ActiveValue<Uuid>,
) -> Result<MusicActiveModel, MusicRepositoryError> {
    Ok(MusicActiveModel {
        id,
        title: ActiveValue::Set(music.title().to_owned()),
        artist: ActiveValue::Set(music.artist().to_owned()),
        bpm: ActiveValue::Set(decimal(*music.bpm())?),
        genre: ActiveValue::Set(0),
        jacket: ActiveValue::Set(music.jacket_image_url().to_owned()),
        registration_date: ActiveValue::Set((*music.registration_date()).into()),
        is_test: ActiveValue::Set(*music.is_test()),
    })
}

pub fn sheet_active_model_for_insert(
    sheet: &Sheet,
) -> Result<SheetActiveModel, MusicRepositoryError> {
    sheet_active_model(
        sheet,
        ActiveValue::Set(parse_uuid(sheet.id())?),
        ActiveValue::Set(parse_uuid(sheet.music_id())?),
    )
}

pub fn sheet_active_model_for_update(
    sheet: &Sheet,
) -> Result<SheetActiveModel, MusicRepositoryError> {
    sheet_active_model(
        sheet,
        ActiveValue::Unchanged(parse_uuid(sheet.id())?),
        ActiveValue::Unchanged(parse_uuid(sheet.music_id())?),
    )
}

fn sheet_active_model(
    sheet: &Sheet,
    id: ActiveValue<Uuid>,
    music_id: ActiveValue<Uuid>,
) -> Result<SheetActiveModel, MusicRepositoryError> {
    let level = sheet.level().components();
    let difficulty = match sheet.difficulty() {
        Difficulty::Easy => DbDifficulty::Easy,
        Difficulty::Normal => DbDifficulty::Normal,
        Difficulty::Hard => DbDifficulty::Hard,
    };
    Ok(SheetActiveModel {
        id,
        music_id,
        difficulty: ActiveValue::Set(difficulty),
        level: ActiveValue::Set((level.0 * 10 + level.1) as i32),
        notes_designer: ActiveValue::Set(sheet.notes_designer().to_owned()),
    })
}

fn decimal(value: f32) -> Result<Decimal, MusicRepositoryError> {
    value.to_string().parse().map_err(|error| {
        MusicRepositoryError::InternalError(AnyError::msg(format!("invalid BPM: {error}")))
    })
}

fn parse_uuid(value: &str) -> Result<Uuid, MusicRepositoryError> {
    Uuid::parse_str(value)
        .map_err(|error| MusicRepositoryError::InternalError(AnyError::from(error)))
}

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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use domain::entity::{difficulty::Difficulty, genre::Genre, level::Level};
    use sea_orm::ActiveValue;

    use super::*;

    fn music() -> Music {
        Music::new(
            "00000000-0000-0000-0000-000000000001".to_owned(),
            "Song".to_owned(),
            "Artist".to_owned(),
            135.5,
            Genre::ORIGINAL,
            "jacket.png".to_owned(),
            Utc.with_ymd_and_hms(2025, 10, 1, 12, 0, 0).unwrap(),
            false,
        )
    }

    fn sheet() -> Sheet {
        Sheet::new(
            "00000000-0000-0000-0000-000000000002".to_owned(),
            "00000000-0000-0000-0000-000000000001".to_owned(),
            Difficulty::Hard,
            Level::new(14, 7).unwrap(),
            "Designer".to_owned(),
        )
    }

    #[test]
    fn insert_active_models_set_identity_fields() {
        let music_model = music_active_model_for_insert(&music()).unwrap();
        let sheet_model = sheet_active_model_for_insert(&sheet()).unwrap();

        assert!(matches!(music_model.id, ActiveValue::Set(_)));
        assert!(matches!(sheet_model.id, ActiveValue::Set(_)));
        assert!(matches!(sheet_model.music_id, ActiveValue::Set(_)));
    }

    #[test]
    fn update_active_models_keep_identity_fields_unchanged() {
        let music_model = music_active_model_for_update(&music()).unwrap();
        let sheet_model = sheet_active_model_for_update(&sheet()).unwrap();

        assert!(matches!(music_model.id, ActiveValue::Unchanged(_)));
        assert!(matches!(sheet_model.id, ActiveValue::Unchanged(_)));
        assert!(matches!(sheet_model.music_id, ActiveValue::Unchanged(_)));
    }
}
