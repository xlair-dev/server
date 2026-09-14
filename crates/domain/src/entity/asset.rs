use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Asset {
    key: String,
    updated_at: DateTime<Utc>,
}

impl Asset {
    pub fn new(key: String, updated_at: DateTime<Utc>) -> Self {
        Self { key, updated_at }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
