use anyhow::Error as AnyError;
use domain::{
    entity::music::Music,
    repository::music::{MusicRepositoryError, MusicWithSheets},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use tracing::error;

use crate::entities;

fn decimal(value: f32) -> Result<sea_orm::prelude::Decimal, MusicRepositoryError> {
    value.to_string().parse().map_err(|error| {
        MusicRepositoryError::InternalError(AnyError::msg(format!("invalid BPM: {error}")))
    })
}

fn music_active_model(
    music: &Music,
) -> Result<entities::musics::ActiveModel, MusicRepositoryError> {
    Ok(entities::musics::ActiveModel {
        id: Set(uuid::Uuid::parse_str(music.id())
            .map_err(|error| MusicRepositoryError::InternalError(AnyError::from(error)))?),
        title: Set(music.title().to_owned()),
        artist: Set(music.artist().to_owned()),
        bpm: Set(decimal(*music.bpm())?),
        genre: Set(0),
        jacket: Set(music.jacket_image_url().to_owned()),
        registration_date: Set((*music.registration_date()).into()),
        is_test: Set(*music.is_test()),
    })
}

fn sheet_active_model(
    sheet: &domain::entity::sheet::Sheet,
) -> Result<entities::sheets::ActiveModel, MusicRepositoryError> {
    let id = uuid::Uuid::parse_str(sheet.id())
        .map_err(|error| MusicRepositoryError::InternalError(AnyError::from(error)))?;
    let music_id = uuid::Uuid::parse_str(sheet.music_id())
        .map_err(|error| MusicRepositoryError::InternalError(AnyError::from(error)))?;
    let level = sheet.level().components();
    let difficulty = match sheet.difficulty() {
        domain::entity::difficulty::Difficulty::Easy => {
            entities::sea_orm_active_enums::Difficulty::Easy
        }
        domain::entity::difficulty::Difficulty::Normal => {
            entities::sea_orm_active_enums::Difficulty::Normal
        }
        domain::entity::difficulty::Difficulty::Hard => {
            entities::sea_orm_active_enums::Difficulty::Hard
        }
    };
    Ok(entities::sheets::ActiveModel {
        id: Set(id),
        music_id: Set(music_id),
        difficulty: Set(difficulty),
        level: Set((level.0 * 10 + level.1) as i32),
        notes_designer: Set(sheet.notes_designer().to_owned()),
    })
}

pub async fn insert_with_sheets(
    db: &DbConn,
    music: MusicWithSheets,
) -> Result<MusicWithSheets, MusicRepositoryError> {
    let txn = db.begin().await.map_err(internal)?;
    let music_model = music_active_model(&music.music)?;
    let result = async {
        music_model.insert(&txn).await.map_err(internal)?;
        for sheet in &music.sheets {
            sheet_active_model(sheet)?
                .insert(&txn)
                .await
                .map_err(internal)?;
        }
        Ok::<_, MusicRepositoryError>(())
    }
    .await;
    if let Err(error) = result {
        let _ = txn.rollback().await;
        return Err(error);
    }
    txn.commit().await.map_err(internal)?;
    Ok(music)
}

pub async fn update_with_sheets(
    db: &DbConn,
    music: MusicWithSheets,
) -> Result<MusicWithSheets, MusicRepositoryError> {
    let txn = db.begin().await.map_err(internal)?;
    let music_id = uuid::Uuid::parse_str(music.music.id())
        .map_err(|error| MusicRepositoryError::InternalError(AnyError::from(error)))?;
    let result = async {
        let existing = entities::sheets::Entity::find()
            .filter(entities::sheets::Column::MusicId.eq(music_id))
            .all(&txn)
            .await
            .map_err(internal)?;
        let existing_ids: std::collections::HashSet<_> =
            existing.iter().map(|sheet| sheet.id).collect();
        let requested_ids: std::collections::HashSet<_> = music
            .sheets
            .iter()
            .map(|sheet| {
                uuid::Uuid::parse_str(sheet.id())
                    .map_err(|error| MusicRepositoryError::InternalError(AnyError::from(error)))
            })
            .collect::<Result<_, _>>()?;
        if existing_ids != requested_ids || existing.len() != 3 {
            return Err(MusicRepositoryError::InternalError(AnyError::msg(
                "music must have exactly three existing sheets",
            )));
        }
        music_active_model(&music.music)?
            .update(&txn)
            .await
            .map_err(internal)?;
        for sheet in &music.sheets {
            sheet_active_model(sheet)?
                .update(&txn)
                .await
                .map_err(internal)?;
        }
        Ok::<_, MusicRepositoryError>(())
    }
    .await;
    if let Err(error) = result {
        let _ = txn.rollback().await;
        return Err(error);
    }
    txn.commit().await.map_err(internal)?;
    Ok(music)
}

fn internal(error: sea_orm::DbErr) -> MusicRepositoryError {
    error!(error = %error, "Failed to write music metadata");
    MusicRepositoryError::InternalError(AnyError::from(error))
}
