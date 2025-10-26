use anyhow::Error as AnyError;
use domain::{entity::user::User, repository::user::UserRepositoryError};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbConn, EntityTrait, QueryFilter, sea_query::Expr,
};
use tracing::{debug, error, info};

use super::query::parse_user_uuid;
use crate::entities;

pub async fn increment_credits(db: &DbConn, user_id: &str) -> Result<u32, UserRepositoryError> {
    let uuid = parse_user_uuid(user_id)?;

    let update = entities::users::Entity::update_many()
        .col_expr(
            entities::users::Column::Credits,
            Expr::col(entities::users::Column::Credits).add(1),
        )
        .filter(entities::users::Column::Id.eq(uuid))
        .exec(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to increment user credits");
            UserRepositoryError::InternalError(AnyError::from(err))
        })?;

    if update.rows_affected == 0 {
        debug!("User not found for supplied id");
        return Err(UserRepositoryError::NotFound(user_id.to_owned()));
    }

    let model = entities::users::Entity::find_by_id(uuid)
        .one(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to fetch user after increment");
            UserRepositoryError::InternalError(AnyError::from(err))
        })?
        .ok_or_else(|| {
            debug!("User disappeared after increment");
            UserRepositoryError::NotFound(user_id.to_owned())
        })?;

    info!(user_id = %model.id, credits = model.credits, "User credits incremented successfully");

    Ok(model.credits as u32)
}

pub async fn save_user(db: &DbConn, user: User) -> Result<User, UserRepositoryError> {
    let uuid = parse_user_uuid(user.id())?;

    let mut active: entities::users::ActiveModel = user.into();
    active.id = ActiveValue::Set(uuid);

    let model = active.update(db).await.map_err(|err| {
        error!(error = %err, "Failed to update user");
        UserRepositoryError::InternalError(AnyError::from(err))
    })?;

    info!(user_id = %model.id, "User updated successfully");
    Ok(model.into())
}
