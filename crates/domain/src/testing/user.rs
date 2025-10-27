use chrono::{DateTime, Utc};

use crate::entity::{rating::Rating, user::User};

use super::datetime::{later_timestamp, sample_timestamp};

pub struct UserSample {
    pub id: &'static str,
    pub card: &'static str,
    pub display_name: &'static str,
    pub rating: u32,
    pub xp: u32,
    pub credits: u32,
}

impl UserSample {
    /// Builds a `User` aggregate based on this sample and the supplied metadata.
    pub fn build(&self, created_at: DateTime<Utc>, is_admin: bool) -> User {
        User::new(
            self.id.to_owned(),
            self.card.to_owned(),
            self.display_name.to_owned(),
            Rating::new(self.rating),
            self.xp,
            self.credits,
            is_admin,
            created_at,
        )
    }
}

pub const USER1: UserSample = UserSample {
    id: "550e8400-e29b-41d4-a716-446655440000",
    card: "CARD-001",
    display_name: "Alice",
    rating: 1500,
    xp: 200,
    credits: 300,
};

pub const USER2: UserSample = UserSample {
    id: "550e8400-e29b-41d4-a716-446655440111",
    card: "CARD-002",
    display_name: "Bob",
    rating: 2500,
    xp: 999,
    credits: 123,
};

pub const USER3: UserSample = UserSample {
    id: "550e8400-e29b-41d4-a716-446655440222",
    card: "CARD-003",
    display_name: "Carol",
    rating: 1800,
    xp: 123,
    credits: 456,
};

pub fn created_at1() -> DateTime<Utc> {
    sample_timestamp()
}

pub fn created_at2() -> DateTime<Utc> {
    later_timestamp()
}
