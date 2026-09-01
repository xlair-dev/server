use anyhow::Error as AnyError;
use domain::repository::music::{MusicRepositoryError, MusicWithSheets};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, TransactionTrait};
use tracing::error;

use super::write_adapter::{
    music_active_model_for_insert, music_active_model_for_update, sheet_active_model_for_insert,
    sheet_active_model_for_update,
};
use crate::entities;

pub async fn insert_with_sheets(
    db: &DbConn,
    music: MusicWithSheets,
) -> Result<MusicWithSheets, MusicRepositoryError> {
    let txn = db.begin().await.map_err(internal)?;
    let music_model = music_active_model_for_insert(&music.music)?;
    let result = async {
        music_model.insert(&txn).await.map_err(internal)?;
        for sheet in &music.sheets {
            sheet_active_model_for_insert(sheet)?
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
        music_active_model_for_update(&music.music)?
            .update(&txn)
            .await
            .map_err(internal)?;
        for sheet in &music.sheets {
            sheet_active_model_for_update(sheet)?
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
