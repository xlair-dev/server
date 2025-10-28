use anyhow::Error as AnyError;
use domain::repository::music::{MusicRepositoryError, MusicWithSheets};
use sea_orm::{DbConn, EntityTrait, QueryOrder};
use tracing::{debug, error};

use super::adapter;
use crate::entities;

/// Collects every music alongside its sheets.
///
/// # Implicit dependencies
/// - Relies on the `fk_sheets_music` foreign key relation in the database to ensure that each sheet
///   references an existing music entry.
pub async fn list_with_sheets(db: &DbConn) -> Result<Vec<MusicWithSheets>, MusicRepositoryError> {
    debug!("Querying musics with related sheets");
    let models = entities::musics::Entity::find()
        .order_by_asc(entities::musics::Column::RegistrationDate)
        .find_with_related(entities::sheets::Entity)
        .all(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to fetch musics");
            MusicRepositoryError::InternalError(AnyError::from(err))
        })?;

    let mut musics = Vec::with_capacity(models.len());
    for (music_model, sheet_models) in models {
        let music = adapter::convert_music(music_model)?;
        let sheets = adapter::convert_sheets(sheet_models)?;
        musics.push(MusicWithSheets::new(music, sheets));
    }

    Ok(musics)
}
