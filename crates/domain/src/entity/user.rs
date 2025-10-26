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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_temporary_initializes_with_defaults() {
        let user = User::new_temporary("CARD-123".to_owned(), "Alice".to_owned());

        assert!(user.id().is_empty());
        assert_eq!(user.card(), "CARD-123");
        assert_eq!(user.display_name(), "Alice");
        assert_eq!(user.rating().value(), 0);
        assert_eq!(*user.xp(), 0);
        assert_eq!(*user.credits(), 0);
        assert!(!user.is_admin());
    }

    #[test]
    fn new_preserves_provided_fields() {
        let timestamp = chrono::NaiveDate::from_ymd_opt(2025, 10, 21)
            .unwrap()
            .and_hms_opt(8, 30, 0)
            .unwrap();

        let user = User::new(
            "user-id".to_owned(),
            "CARD-456".to_owned(),
            "Bob".to_owned(),
            Rating::new(1234),
            100,
            50,
            true,
            timestamp,
        );

        assert_eq!(user.id(), "user-id");
        assert_eq!(user.card(), "CARD-456");
        assert_eq!(user.display_name(), "Bob");
        assert_eq!(user.rating().value(), 1234);
        assert_eq!(*user.xp(), 100);
        assert_eq!(*user.credits(), 50);
        assert!(user.is_admin());
        assert_eq!(*user.created_at(), timestamp);
    }
}
