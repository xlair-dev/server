use std::convert::TryFrom;

use anyhow::Error as AnyError;
use bigdecimal::{Signed, ToPrimitive};
use domain::{entity::user::User, repository::user::UserRepositoryError};
use sea_orm::{
    ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bigdecimal::BigDecimal;
    use sea_orm::{DatabaseBackend, MockDatabase, sea_query::Value};

    use super::*;

    fn decimal_row(label: &str, value: Option<i64>) -> BTreeMap<String, Value> {
        let mapped_value = value
            .map(|v| Value::BigDecimal(Some(Box::new(BigDecimal::from(v)))))
            .unwrap_or(Value::BigDecimal(None));
        BTreeMap::from([(label.to_owned(), mapped_value)])
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
}
