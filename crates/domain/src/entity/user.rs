use chrono::NaiveDateTime;
use getset::{Getters, Setters};

use super::rating::Rating;

#[derive(Debug, Getters, Setters)]
pub struct User {
    #[getset(get = "pub")]
    id: String,
    #[getset(get = "pub")]
    card: String,
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
        card: String,
        display_name: String,
        rating: Rating,
        xp: u32,
        credits: u32,
        is_admin: bool,
        created_at: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            card,
            display_name,
            rating,
            xp,
            credits,
            is_admin,
            created_at,
        }
    }

    pub fn new_temporary(card: String, display_name: String) -> Self {
        Self {
            id: "".to_string(),
            card,
            display_name,
            rating: Rating::default(),
            xp: 0,
            credits: 0,
            is_admin: false,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}
