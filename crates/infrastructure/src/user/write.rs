use std::convert::TryFrom;

use anyhow::Error as AnyError;
use chrono::Utc;
use domain::{
    entity::{user::User, user_play_option::UserPlayOption},
    repository::user::UserRepositoryError,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbConn, EntityTrait, QueryFilter,
    sea_query::{Expr, OnConflict},
};
use tracing::{debug, error, info};

use super::adapter::{convert_user_insert_error, parse_user_uuid};
use crate::entities;

pub async fn create_user(db: &DbConn, user: User) -> Result<User, UserRepositoryError> {
    let card_id = user.card().to_owned();
    let db_user: entities::users::ActiveModel = user.into();

    let db_user_model = db_user
        .insert(db)
        .await
        .map_err(|err| convert_user_insert_error(err, &card_id))?;

    debug!(user_id = %db_user_model.id, "User persisted by repository");
    User::try_from(db_user_model)
}

pub async fn increment_credits(db: &DbConn, user_id: &str) -> Result<(), UserRepositoryError> {
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
            error!(error = %err, user_id = %uuid, "Failed to increment user credits");
            UserRepositoryError::InternalError(AnyError::from(err))
        })?;

    if update.rows_affected == 0 {
        debug!("User not found for supplied id");
        return Err(UserRepositoryError::NotFound(user_id.to_owned()));
    }

    debug!(user_id = %uuid, "User credits incremented successfully");

    Ok(())
}

pub async fn save_user(db: &DbConn, user: User) -> Result<User, UserRepositoryError> {
    let uuid = parse_user_uuid(user.id())?;

    let mut active: entities::users::ActiveModel = user.into();
    active.id = ActiveValue::Set(uuid);

    let model = active.update(db).await.map_err(|err| {
        error!(error = %err, user_id = %uuid, "Failed to update user");
        UserRepositoryError::InternalError(AnyError::from(err))
    })?;

    info!(user_id = %model.id, "User updated successfully");
    User::try_from(model)
}

pub async fn save_play_option(
    db: &DbConn,
    mut option: UserPlayOption,
) -> Result<UserPlayOption, UserRepositoryError> {
    let uuid = parse_user_uuid(option.user_id())?;
    option.set_updated_at(Utc::now());

    let mut active: entities::user_play_options::ActiveModel = option.into();
    active.user_id = ActiveValue::Set(uuid);

    let model = entities::user_play_options::Entity::insert(active)
        .on_conflict(
            OnConflict::column(entities::user_play_options::Column::UserId)
                .update_columns([
                    entities::user_play_options::Column::NoteSpeed,
                    entities::user_play_options::Column::JudgmentOffset,
                    entities::user_play_options::Column::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec_with_returning(db)
        .await
        .map_err(|err| {
            error!(
                error = %err,
                user_id = %uuid,
                "Failed to persist user play option"
            );
            UserRepositoryError::InternalError(AnyError::from(err))
        })?;

    info!(user_id = %uuid, "User play option persisted successfully");
    UserPlayOption::try_from(model)
}
