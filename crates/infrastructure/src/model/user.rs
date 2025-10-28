use anyhow::Error as AnyError;
use chrono::Utc;
use domain::{
    entity::{rating::Rating, user::User},
    repository::user::UserRepositoryError,
};
use sea_orm::{ActiveValue, prelude::Uuid};

use crate::entities::users::{ActiveModel as UserActiveModel, Model as UserModel};

/// Converts domain user entity to database model.
///
/// # Panics
/// Panics if rating value exceeds i32 range. This should never happen in practice
/// as rating values are constrained by business logic (typically under 10,000).
impl From<User> for UserModel {
    fn from(domain_user: User) -> Self {
        let id = Uuid::parse_str(domain_user.id()).unwrap_or_else(|_| Uuid::nil());
        Self {
            id,
            card: domain_user.card().to_owned(),
            display_name: domain_user.display_name().to_owned(),
            rating: i32::try_from(domain_user.rating().value())
                .expect("rating exceeds database range"),
            xp: domain_user.xp().to_owned() as i64,
            credits: domain_user.credits().to_owned() as i64,
            is_public: *domain_user.is_public(),
            is_admin: *domain_user.is_admin(),
            created_at: (*domain_user.created_at()).into(),
            updated_at: Utc::now().into(),
        }
    }
}

/// Converts database user model to domain entity.
///
/// # Errors
/// Returns `InternalError` if the database contains invalid data (negative rating).
/// This conversion assumes database integrity constraints ensure valid data.
impl std::convert::TryFrom<UserModel> for User {
    type Error = UserRepositoryError;

    fn try_from(db_user: UserModel) -> Result<Self, Self::Error> {
        let id = db_user.id.to_string();
        let created_at = db_user.created_at.with_timezone(&chrono::Utc);

        let rating_value = u32::try_from(db_user.rating).map_err(|err| {
            tracing::warn!(error = %err, value = db_user.rating, "Rating from database must be non-negative");
            UserRepositoryError::InternalError(AnyError::from(err))
        })?;
        let rating = Rating::new(rating_value);

        Ok(Self::new(
            id,
            db_user.card,
            db_user.display_name,
            rating,
            db_user.xp as u32,
            db_user.credits as u32,
            db_user.is_public,
            db_user.is_admin,
            created_at,
        ))
    }
}

/// Converts domain user entity to database active model for updates.
///
/// # Panics
/// Panics if rating value exceeds i32 range. This should never happen in practice
/// as rating values are constrained by business logic (typically under 10,000).
impl From<User> for UserActiveModel {
    fn from(domain_user: User) -> Self {
        // when inserting/updating via ActiveModel we prefer to set fields explicitly
        let db_user_id = if domain_user.id().is_empty() {
            ActiveValue::NotSet
        } else {
            ActiveValue::Set(Uuid::parse_str(domain_user.id()).unwrap_or_else(|_| Uuid::nil()))
        };

        let db_user_created_at = if domain_user.id().is_empty() {
            ActiveValue::NotSet
        } else {
            ActiveValue::Set((*domain_user.created_at()).into())
        };

        UserActiveModel {
            id: db_user_id,
            card: ActiveValue::Set(domain_user.card().to_owned()),
            display_name: ActiveValue::Set(domain_user.display_name().to_owned()),
            rating: ActiveValue::Set(
                i32::try_from(domain_user.rating().value()).expect("rating exceeds database range"),
            ),
            xp: ActiveValue::Set(domain_user.xp().to_owned() as i64),
            credits: ActiveValue::Set(domain_user.credits().to_owned() as i64),
            is_public: ActiveValue::Set(*domain_user.is_public()),
            is_admin: ActiveValue::Set(*domain_user.is_admin()),
            created_at: db_user_created_at,
            updated_at: ActiveValue::NotSet,
        }
    }
}

#[cfg(test)]
mod tests {
    use domain::testing::user::{USER2, USER3, created_at1};
    use sea_orm::prelude::Uuid;

    use super::*;
    use crate::entities::users::Model as RawUserModel;

    #[test]
    fn user_model_from_domain_sets_uuid_and_timestamps() {
        let created_at = created_at1();
        let user = USER2.build(true, true, created_at);

        let model: UserModel = user.into();

        assert_eq!(model.id, Uuid::parse_str(USER2.id).unwrap());
        assert_eq!(model.card, USER2.card);
        assert_eq!(model.display_name, USER2.display_name);
        assert_eq!(model.rating, USER2.rating as i32);
        assert_eq!(model.xp, USER2.xp as i64);
        assert_eq!(model.credits, USER2.credits as i64);
        assert!(model.is_public);
        assert!(model.is_admin);
        assert_eq!(model.created_at, created_at);
    }

    #[test]
    fn domain_user_from_model_preserves_scalar_fields() {
        let created_at = chrono::Utc::now();
        let model = RawUserModel {
            id: Uuid::parse_str(USER3.id).unwrap(),
            card: USER3.card.to_owned(),
            display_name: USER3.display_name.to_owned(),
            rating: USER3.rating as i32,
            xp: USER3.xp as i64,
            credits: USER3.credits as i64,
            is_public: false,
            is_admin: false,
            created_at: created_at.into(),
            updated_at: created_at.into(),
        };

        let user: User = User::try_from(model.clone()).unwrap();

        let model_id = model.id.to_string();
        assert_eq!(user.id(), &model_id);
        assert_eq!(user.card(), USER3.card);
        assert_eq!(user.display_name(), USER3.display_name);
        assert_eq!(user.rating().value(), USER3.rating);
        assert_eq!(*user.xp(), USER3.xp);
        assert_eq!(*user.credits(), USER3.credits);
        assert!(!user.is_public());
        assert!(!user.is_admin());
        assert_eq!(*user.created_at(), created_at);
    }

    #[test]
    fn active_model_from_temporary_user_leaves_identity_unset() {
        let user = User::new_temporary(USER2.card.to_owned(), USER2.display_name.to_owned());

        let active: UserActiveModel = user.into();

        assert!(matches!(active.id, ActiveValue::NotSet));
        assert!(matches!(active.created_at, ActiveValue::NotSet));

        if let ActiveValue::Set(card) = &active.card {
            assert_eq!(card, USER2.card);
        } else {
            panic!("expected card to be set");
        }

        if let ActiveValue::Set(name) = &active.display_name {
            assert_eq!(name, USER2.display_name);
        } else {
            panic!("expected display_name to be set");
        }
    }
}
