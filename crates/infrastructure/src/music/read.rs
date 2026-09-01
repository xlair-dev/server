use anyhow::Error as AnyError;
use chrono::Utc;
use domain::repository::music::{
    MusicListCursor, MusicListPage, MusicRepositoryError, MusicWithSheets,
};
use sea_orm::{ColumnTrait, Condition, DbConn, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
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

/// Loads one ordered page of music and its sheets directly from the database.
pub async fn list_with_sheets_page(
    db: &DbConn,
    cursor: Option<MusicListCursor>,
    limit: u64,
) -> Result<MusicListPage, MusicRepositoryError> {
    debug!(
        limit,
        has_cursor = cursor.is_some(),
        "Querying a page of musics with related sheets"
    );

    let mut query = entities::musics::Entity::find();
    if let Some(cursor) = cursor {
        let cursor_id = uuid::Uuid::parse_str(&cursor.id)
            .map_err(|error| MusicRepositoryError::InternalError(AnyError::from(error)))?;
        query = query.filter(
            Condition::any()
                .add(entities::musics::Column::RegistrationDate.gt(cursor.registration_date))
                .add(
                    Condition::all()
                        .add(
                            entities::musics::Column::RegistrationDate.eq(cursor.registration_date),
                        )
                        .add(entities::musics::Column::Id.gt(cursor_id)),
                ),
        );
    }

    let mut models = query
        .order_by_asc(entities::musics::Column::RegistrationDate)
        .order_by_asc(entities::musics::Column::Id)
        .limit(limit + 1)
        .find_with_related(entities::sheets::Entity)
        .all(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to fetch a page of musics");
            MusicRepositoryError::InternalError(AnyError::from(err))
        })?;

    let next_cursor = if models.len() > limit as usize {
        models.pop().map(|(model, _)| MusicListCursor {
            registration_date: model.registration_date.with_timezone(&Utc),
            id: model.id.to_string(),
        })
    } else {
        None
    };

    let mut items = Vec::with_capacity(models.len());
    for (music_model, sheet_models) in models {
        let music = adapter::convert_music(music_model)?;
        let sheets = adapter::convert_sheets(sheet_models)?;
        items.push(MusicWithSheets::new(music, sheets));
    }

    Ok(MusicListPage { items, next_cursor })
}

/// Loads one music entry with all related sheets.
pub async fn find_with_sheets(
    db: &DbConn,
    music_id: &str,
) -> Result<MusicWithSheets, MusicRepositoryError> {
    let id = uuid::Uuid::parse_str(music_id)
        .map_err(|error| MusicRepositoryError::InternalError(AnyError::from(error)))?;
    let result = entities::musics::Entity::find_by_id(id)
        .find_with_related(entities::sheets::Entity)
        .all(db)
        .await
        .map_err(|err| {
            error!(error = %err, music_id, "Failed to fetch music by id");
            MusicRepositoryError::InternalError(AnyError::from(err))
        })?
        .into_iter()
        .next();

    let Some((music_model, sheet_models)) = result else {
        return Err(MusicRepositoryError::NotFound(music_id.to_owned()));
    };

    Ok(MusicWithSheets::new(
        adapter::convert_music(music_model)?,
        adapter::convert_sheets(sheet_models)?,
    ))
}
