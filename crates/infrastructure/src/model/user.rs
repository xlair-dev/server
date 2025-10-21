use chrono::{NaiveDateTime, TimeZone, Utc};
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
            created_at: Utc.from_utc_datetime(domain_user.created_at()).into(),
            updated_at: Utc::now().into(),
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
            ActiveValue::Set(Utc.from_utc_datetime(domain_user.created_at()).into())
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
    use domain::testing::{
        datetime::later_timestamp,
        user::{USER2, USER3, created_at1},
    };
    use sea_orm::prelude::Uuid;

    #[test]
    fn user_model_from_domain_sets_uuid_and_timestamps() {
        let created_at = created_at1();
        let user = USER2.build(created_at, true);

        let model: UserModel = user.into();

        assert_eq!(model.id, Uuid::parse_str(USER2.id).unwrap());
        assert_eq!(model.card, USER2.card);
        assert_eq!(model.display_name, USER2.display_name);
        assert_eq!(model.rating, USER2.rating as i32);
        assert_eq!(model.xp, USER2.xp as i64);
        assert_eq!(model.credits, USER2.credits as i64);
        assert!(model.is_admin);
        assert_eq!(model.created_at.naive_utc(), created_at);
    }

    #[test]
    fn domain_user_from_model_preserves_scalar_fields() {
        let created_at_naive = later_timestamp();
        let created_at = chrono::Utc.from_utc_datetime(&created_at_naive);
        let model = RawUserModel {
            id: Uuid::parse_str(USER3.id).unwrap(),
            card: USER3.card.to_owned(),
            display_name: USER3.display_name.to_owned(),
            rating: USER3.rating as i32,
            xp: USER3.xp as i64,
            credits: USER3.credits as i64,
            is_admin: false,
            created_at: created_at.into(),
            updated_at: created_at.into(),
        };

        let user: User = model.clone().into();

        let model_id = model.id.to_string();
        assert_eq!(user.id(), &model_id);
        assert_eq!(user.card(), USER3.card);
        assert_eq!(user.display_name(), USER3.display_name);
        assert_eq!(user.rating().value(), USER3.rating);
        assert_eq!(*user.xp(), USER3.xp);
        assert_eq!(*user.credits(), USER3.credits);
        assert!(!user.is_admin());
        assert_eq!(*user.created_at(), created_at_naive);
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
