use std::convert::TryFrom;

use anyhow::Error as AnyError;
use bigdecimal::{Signed, ToPrimitive};
use domain::{
    entity::{user::User, user_play_option::UserPlayOption},
    repository::user::UserRepositoryError,
};
use sea_orm::{
    ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::{Alias, Expr},
    sqlx::types::BigDecimal,
};
use tracing::{debug, error, info};

use crate::{entities, user::adapter::parse_user_uuid};

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
        Ok(Some(User::try_from(model)?))
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

    model.map(User::try_from).transpose()
}

pub async fn find_play_option(
    db: &DbConn,
    user_id: &str,
) -> Result<Option<UserPlayOption>, UserRepositoryError> {
    let uuid = parse_user_uuid(user_id)?;

    debug!(user_id = %uuid, "Querying user play option via SeaORM");
    let model = entities::user_play_options::Entity::find_by_id(uuid)
        .one(db)
        .await
        .map_err(|err| {
            error!(error = %err, user_id = %uuid, "Failed to query user play option");
            UserRepositoryError::InternalError(AnyError::from(err))
        })?;

    match model {
        Some(model) => {
            info!(user_id = %uuid, "User play option fetched successfully");
            UserPlayOption::try_from(model).map(Some)
        }
        None => {
            debug!(user_id = %uuid, "User play option not found");
            Ok(None)
        }
    }
}

pub async fn count_all(db: &DbConn) -> Result<u64, UserRepositoryError> {
    debug!("Counting users via SeaORM");
    let count = entities::users::Entity::find()
        .count(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to count users");
            UserRepositoryError::InternalError(AnyError::from(err))
        })?;
    info!(total_users = count, "User count fetched successfully");
    Ok(count)
}

/// Aggregates user credits across all accounts. Casts SUM(...) to NUMERIC so
/// Postgres always returns a consistent type for decoding.
pub async fn sum_credits(db: &DbConn) -> Result<u64, UserRepositoryError> {
    debug!("Summing user credits via SeaORM");
    let sum = entities::users::Entity::find()
        .select_only()
        .column_as(
            Expr::col(entities::users::Column::Credits)
                .sum()
                .cast_as(Alias::new("numeric")),
            "sum",
        )
        .into_tuple::<Option<BigDecimal>>()
        .one(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to sum user credits");
            UserRepositoryError::InternalError(AnyError::from(err))
        })?
        .flatten()
        .unwrap_or_else(|| BigDecimal::from(0u8));

    if sum.is_negative() {
        let err = AnyError::msg("Database returned negative user credit sum");
        error!("User credit sum returned negative value");
        return Err(UserRepositoryError::InternalError(err));
    }

    let sum = sum.to_u64().ok_or_else(|| {
        let err = AnyError::msg("User credit sum cannot fit in u64");
        error!("User credit sum overflowed u64 conversion");
        UserRepositoryError::InternalError(err)
    })?;

    info!(total_credits = sum, "User credits summed successfully");
    Ok(sum)
}

pub async fn public_users_by_rating(
    db: &DbConn,
    limit: u64,
) -> Result<Vec<User>, UserRepositoryError> {
    debug!(limit, "Querying public users by rating via SeaORM");
    let models = entities::users::Entity::find()
        .filter(entities::users::Column::IsPublic.eq(true))
        .order_by_desc(entities::users::Column::Rating)
        .order_by_asc(entities::users::Column::DisplayName)
        .limit(limit)
        .all(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to query public users by rating");
            UserRepositoryError::InternalError(AnyError::from(err))
        })?;

    let mut result = Vec::with_capacity(models.len());
    for model in models {
        let user = User::try_from(model)?;
        result.push(user);
    }

    info!(
        count = result.len(),
        "Public users by rating fetched successfully"
    );
    Ok(result)
}

pub async fn public_users_by_xp(db: &DbConn, limit: u64) -> Result<Vec<User>, UserRepositoryError> {
    debug!(limit, "Querying public users by XP via SeaORM");
    let models = entities::users::Entity::find()
        .filter(entities::users::Column::IsPublic.eq(true))
        .order_by_desc(entities::users::Column::Xp)
        .order_by_desc(entities::users::Column::Rating)
        .order_by_asc(entities::users::Column::DisplayName)
        .limit(limit)
        .all(db)
        .await
        .map_err(|err| {
            error!(error = %err, "Failed to query public users by XP");
            UserRepositoryError::InternalError(AnyError::from(err))
        })?;

    let mut result = Vec::with_capacity(models.len());
    for model in models {
        let user = User::try_from(model)?;
        result.push(user);
    }

    info!(
        count = result.len(),
        "Public users by XP fetched successfully"
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bigdecimal::BigDecimal;
    use chrono::{TimeZone, Utc};
    use sea_orm::{DatabaseBackend, MockDatabase, prelude::Uuid, sea_query::Value};

    use super::*;
    use crate::entities;

    fn decimal_row(label: &str, value: Option<i64>) -> BTreeMap<String, Value> {
        let mapped_value = value
            .map(|v| Value::BigDecimal(Some(Box::new(BigDecimal::from(v)))))
            .unwrap_or(Value::BigDecimal(None));
        BTreeMap::from([(label.to_owned(), mapped_value)])
    }

    fn user_model(
        id: Uuid,
        card: &str,
        display_name: &str,
        rating: i32,
        xp: i64,
        credits: i64,
        is_public: bool,
    ) -> entities::users::Model {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        entities::users::Model {
            id,
            card: card.to_owned(),
            display_name: display_name.to_owned(),
            rating,
            xp,
            credits,
            is_public,
            is_admin: false,
            created_at: timestamp.into(),
            updated_at: timestamp.into(),
        }
    }

    #[tokio::test]
    async fn sum_credits_handles_numeric_rows() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![decimal_row("sum", Some(42_000))]])
            .into_connection();

        let result = sum_credits(&db).await.unwrap();
        assert_eq!(result, 42_000);
    }

    #[tokio::test]
    async fn sum_credits_defaults_to_zero() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![decimal_row("sum", None)]])
            .into_connection();

        let result = sum_credits(&db).await.unwrap();
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn public_users_by_rating_returns_users() {
        let user_id = Uuid::parse_str("dddddddd-dddd-dddd-dddd-dddddddddddd").expect("valid uuid");
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model(
                user_id, "card-1", "Alice", 1800, 42, 10, true,
            )]])
            .into_connection();

        let result = public_users_by_rating(&db, 1).await.unwrap();

        assert_eq!(result.len(), 1);
        let user = &result[0];
        assert_eq!(user.id(), &user_id.to_string());
        assert_eq!(user.display_name(), "Alice");
        assert_eq!(user.rating().value(), 1800);
    }

    #[tokio::test]
    async fn public_users_by_xp_returns_users() {
        let user_id = Uuid::parse_str("eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee").expect("valid uuid");
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model(
                user_id, "card-2", "Bob", 1700, 123, 20, true,
            )]])
            .into_connection();

        let result = public_users_by_xp(&db, 1).await.unwrap();

        assert_eq!(result.len(), 1);
        let user = &result[0];
        assert_eq!(user.id(), &user_id.to_string());
        assert_eq!(user.display_name(), "Bob");
        assert_eq!(user.xp(), &123);
    }
}
