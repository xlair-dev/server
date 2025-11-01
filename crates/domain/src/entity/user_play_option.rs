use chrono::{DateTime, Utc};
use getset::{Getters, Setters};

/// Domain representation of per-user play options.
#[derive(Debug, Clone, Getters, Setters)]
pub struct UserPlayOption {
    #[getset(get = "pub")]
    user_id: String,
    #[getset(get = "pub", set = "pub")]
    note_speed: f32,
    #[getset(get = "pub", set = "pub")]
    judgment_offset: i32,
    #[getset(get = "pub", set = "pub")]
    updated_at: DateTime<Utc>,
}

impl UserPlayOption {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_id: String,
        note_speed: f32,
        judgment_offset: i32,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            note_speed,
            judgment_offset,
            updated_at,
        }
    }

    /// Builds play options with platform defaults.
    pub fn new_with_defaults(user_id: String, updated_at: DateTime<Utc>) -> Self {
        Self::new(user_id, 1.0, 0, updated_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_defaults_uses_expected_values() {
        let timestamp = chrono::Utc::now();
        let options = UserPlayOption::new_with_defaults("user-id".to_owned(), timestamp);

        assert_eq!(options.user_id(), "user-id");
        assert!((options.note_speed() - 1.0).abs() < f32::EPSILON);
        assert_eq!(*options.judgment_offset(), 0);
        assert_eq!(*options.updated_at(), timestamp);
    }
}
