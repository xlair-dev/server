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
    audio_judgment_timing: i32,
    #[getset(get = "pub", set = "pub")]
    visual_judgment_timing: f32,
    #[getset(get = "pub", set = "pub")]
    mirror: bool,
    #[getset(get = "pub", set = "pub")]
    note_volume: u8,
    #[getset(get = "pub", set = "pub")]
    updated_at: DateTime<Utc>,
}

impl UserPlayOption {
    #[allow(clippy::too_many_arguments)]
    pub fn new(user_id: String, note_speed: f32, updated_at: DateTime<Utc>) -> Self {
        Self {
            user_id,
            note_speed,
            audio_judgment_timing: 0,
            visual_judgment_timing: 0.0,
            mirror: false,
            note_volume: 100,
            updated_at,
        }
    }

    /// Builds play options with platform defaults.
    pub fn new_with_defaults(user_id: String, updated_at: DateTime<Utc>) -> Self {
        Self::new(user_id, 1.0, updated_at)
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
        assert_eq!(*options.audio_judgment_timing(), 0);
        assert_eq!(*options.visual_judgment_timing(), 0.0);
        assert!(!*options.mirror());
        assert_eq!(*options.note_volume(), 100);
        assert_eq!(*options.updated_at(), timestamp);
    }
}
