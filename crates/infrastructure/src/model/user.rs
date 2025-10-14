use domain::entity::rating::Rating;
use domain::entity::user::User;

use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::prelude::Uuid;

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
            rating: domain_user.rating().value() as i32,
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
            Rating::new(db_user.rating as f64),
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
            rating: ActiveValue::Set(domain_user.rating().value() as i32),
            xp: ActiveValue::Set(domain_user.xp().to_owned() as i64),
            credits: ActiveValue::Set(domain_user.credits().to_owned() as i64),
            is_admin: ActiveValue::Set(*domain_user.is_admin()),
            created_at: db_user_created_at,
            updated_at: ActiveValue::NotSet,
        }
    }
}
