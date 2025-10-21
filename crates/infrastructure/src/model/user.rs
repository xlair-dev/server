use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::prelude::Uuid;
use std::convert::TryFrom;

use domain::entity::rating::Rating;
use domain::entity::user::User;

use crate::entities::users::ActiveModel as UserActiveModel;
use crate::entities::users::Model as UserModel;
use sea_orm::ActiveValue;

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
            is_admin: *domain_user.is_admin(),
            created_at: DateTime::<Utc>::from_utc(*domain_user.created_at(), Utc).into(),
            updated_at: DateTime::<Utc>::from_utc(Utc::now().naive_utc(), Utc).into(),
        }
    }
}

impl From<UserModel> for User {
    fn from(db_user: UserModel) -> Self {
        let id = db_user.id.to_string();
        let created_at: NaiveDateTime = db_user.created_at.naive_utc();

        Self::new(
            id,
            db_user.card,
            db_user.display_name,
            Rating::new(u32::try_from(db_user.rating).expect("rating must be non-negative")),
            db_user.xp as u32,
            db_user.credits as u32,
            db_user.is_admin,
            created_at,
        )
    }
}

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
            ActiveValue::Set(DateTime::<Utc>::from_utc(*domain_user.created_at(), Utc).into())
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
            is_admin: ActiveValue::Set(*domain_user.is_admin()),
            created_at: db_user_created_at,
            updated_at: ActiveValue::NotSet,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::users::Model as RawUserModel;
    use chrono::TimeZone;
    use sea_orm::prelude::Uuid;

    #[test]
    fn user_model_from_domain_sets_uuid_and_timestamps() {
        let created_at = chrono::Utc
            .with_ymd_and_hms(2025, 10, 21, 12, 0, 0)
            .unwrap()
            .naive_utc();
        let user = User::new(
            "550e8400-e29b-41d4-a716-446655440000".to_owned(),
            "CARD-100".to_owned(),
            "Alice".to_owned(),
            Rating::new(1500),
            200,
            300,
            true,
            created_at,
        );

        let model: UserModel = user.into();

        assert_eq!(
            model.id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(model.card, "CARD-100");
        assert_eq!(model.display_name, "Alice");
        assert_eq!(model.rating, 1500);
        assert_eq!(model.xp, 200);
        assert_eq!(model.credits, 300);
        assert!(model.is_admin);
        assert_eq!(model.created_at.naive_utc(), created_at);
    }

    #[test]
    fn domain_user_from_model_preserves_scalar_fields() {
        let created_at = chrono::Utc
            .with_ymd_and_hms(2025, 10, 21, 9, 30, 0)
            .unwrap();
        let model = RawUserModel {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655449999").unwrap(),
            card: "CARD-200".to_owned(),
            display_name: "Bob".to_owned(),
            rating: 1800,
            xp: 123,
            credits: 456,
            is_admin: false,
            created_at: created_at.into(),
            updated_at: created_at.into(),
        };

        let user: User = model.clone().into();

        let model_id = model.id.to_string();
        assert_eq!(user.id(), &model_id);
        assert_eq!(user.card(), "CARD-200");
        assert_eq!(user.display_name(), "Bob");
        assert_eq!(user.rating().value(), 1800);
        assert_eq!(*user.xp(), 123);
        assert_eq!(*user.credits(), 456);
        assert!(!user.is_admin());
        assert_eq!(*user.created_at(), created_at.naive_utc());
    }

    #[test]
    fn active_model_from_temporary_user_leaves_identity_unset() {
        let user = User::new_temporary("CARD-300".to_owned(), "Carol".to_owned());

        let active: UserActiveModel = user.into();

        assert!(matches!(active.id, ActiveValue::NotSet));
        assert!(matches!(active.created_at, ActiveValue::NotSet));

        if let ActiveValue::Set(card) = &active.card {
            assert_eq!(card, "CARD-300");
        } else {
            panic!("expected card to be set");
        }

        if let ActiveValue::Set(name) = &active.display_name {
            assert_eq!(name, "Carol");
        } else {
            panic!("expected display_name to be set");
        }
    }
}
