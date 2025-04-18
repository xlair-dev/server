use chrono::NaiveDateTime;
use getset::{Getters, Setters};

use super::rating::Rating;

#[derive(Debug, Getters, Setters)]
pub struct User {
    #[getset(get = "pub")]
    id: String,
    #[getset(get = "pub")]
    access_code: String,
    #[getset(get = "pub")]
    card: String,
    #[getset(get = "pub", set = "pub")]
    auth_id: Option<String>,
    #[getset(get = "pub", set = "pub")]
    user_name: Option<String>,
    #[getset(get = "pub", set = "pub")]
    display_name: String,
    #[getset(get = "pub", set = "pub")]
    rating: Rating,
    #[getset(get = "pub", set = "pub")]
    xp: u32,
    #[getset(get = "pub", set = "pub")]
    credits: u32,
    #[getset(get = "pub")]
    is_admin: bool,
    #[getset(get = "pub")]
    created_at: NaiveDateTime,
}

#[allow(clippy::too_many_arguments)]
impl User {
    pub fn new(
        id: String,
        access_code: String,
        card: String,
        auth_id: Option<String>,
        user_name: Option<String>,
        display_name: String,
        rating: Rating,
        xp: u32,
        credits: u32,
        is_admin: bool,
        created_at: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            access_code,
            card,
            auth_id,
            user_name,
            display_name,
            rating,
            xp,
            credits,
            is_admin,
            created_at,
        }
    }
}
