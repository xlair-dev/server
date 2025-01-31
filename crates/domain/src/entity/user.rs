use chrono::NaiveDateTime;
use getset::{Getters, Setters};

#[derive(Debug, Getters, Setters)]
pub struct User {
    #[getset(get = "pub")]
    id: String,
    #[getset(get = "pub")]
    access_code: String,
    #[getset(get = "pub")]
    card: String,
    #[getset(get = "pub")]
    firebase_uid: String,
    #[getset(get = "pub", set = "pub")]
    user_name: String,
    #[getset(get = "pub", set = "pub")]
    display_name: String,
    #[getset(get = "pub", set = "pub")]
    xp: u32,
    #[getset(get = "pub", set = "pub")]
    credit: u32,
    #[getset(get = "pub")]
    is_admin: bool,
    #[getset(get = "pub")]
    created_at: NaiveDateTime, // TODO: consider using DateTime<Utc>
}

#[allow(clippy::too_many_arguments)]
impl User {
    pub fn new(
        id: String,
        access_code: String,
        card: String,
        firebase_uid: String,
        user_name: String,
        display_name: String,
        xp: u32,
        credit: u32,
        is_admin: bool,
        created_at: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            access_code,
            card,
            firebase_uid,
            user_name,
            display_name,
            xp,
            credit,
            is_admin,
            created_at,
        }
    }
}
