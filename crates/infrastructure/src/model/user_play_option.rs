use chrono::Utc;
use domain::{entity::user_play_option::UserPlayOption, repository::user::UserRepositoryError};
use sea_orm::{ActiveValue, prelude::Uuid};

use crate::entities::user_play_options::{
    ActiveModel as UserPlayOptionActiveModel, Model as UserPlayOptionModel,
};

impl From<UserPlayOption> for UserPlayOptionModel {
    fn from(option: UserPlayOption) -> Self {
        let user_id = Uuid::parse_str(option.user_id()).unwrap_or_else(|_| Uuid::nil());
        Self {
            user_id,
            note_speed: f64::from(*option.note_speed()),
            judgment_offset: *option.judgment_offset(),
            updated_at: (*option.updated_at()).into(),
        }
    }
}

impl std::convert::TryFrom<UserPlayOptionModel> for UserPlayOption {
    type Error = UserRepositoryError;

    fn try_from(model: UserPlayOptionModel) -> Result<Self, Self::Error> {
        let user_id = model.user_id.to_string();
        let note_speed = model.note_speed as f32;

        Ok(Self::new(
            user_id,
            note_speed,
            model.judgment_offset,
            model.updated_at.with_timezone(&Utc),
        ))
    }
}

impl From<UserPlayOption> for UserPlayOptionActiveModel {
    fn from(option: UserPlayOption) -> Self {
        let user_id = if option.user_id().is_empty() {
            ActiveValue::NotSet
        } else {
            ActiveValue::Set(Uuid::parse_str(option.user_id()).unwrap_or_else(|_| Uuid::nil()))
        };

        UserPlayOptionActiveModel {
            user_id,
            note_speed: ActiveValue::Set(f64::from(*option.note_speed())),
            judgment_offset: ActiveValue::Set(*option.judgment_offset()),
            updated_at: ActiveValue::Set((*option.updated_at()).into()),
        }
    }
}
