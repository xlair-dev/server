use anyhow::Error as AnyError;
use domain::{entity::user::User, repository::user::UserRepositoryError};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, prelude::Uuid};
use tracing::{debug, error, info};

use crate::entities;

pub async fn find_by_card(db: &DbConn, card: &str) -> Result<Option<User>, UserRepositoryError> {
    debug!("Querying user via SeaORM");
    let model = entities::users::Entity::find()
        .filter(entities::users::Column::Card.eq(card))
        .one(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to query user");
            UserRepositoryError::InternalError(AnyError::from(err))
        })?;

    if let Some(model) = model {
        info!(user_id = %model.id, "User fetched successfully");
        Ok(Some(model.into()))
    } else {
        debug!("User not found for supplied card");
        Ok(None)
    }
}

pub async fn find_by_id(db: &DbConn, user_id: &str) -> Result<Option<User>, UserRepositoryError> {
    let uuid = parse_user_uuid(user_id)?;

    let model = entities::users::Entity::find_by_id(uuid)
        .one(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to query user by id");
            UserRepositoryError::InternalError(AnyError::from(err))
        })?;

    Ok(model.map(Into::into))
}

pub fn parse_user_uuid(user_id: &str) -> Result<Uuid, UserRepositoryError> {
    Uuid::parse_str(user_id).map_err(|err| {
        debug!(error = %err, "Failed to parse user id");
        UserRepositoryError::NotFound(user_id.to_owned())
    })
}
