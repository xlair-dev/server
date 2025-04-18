use anyhow::Result;
use std::fmt::{Display, Formatter, Result as FmtResult};
use thiserror::Error;

#[derive(Debug)]
pub struct Level(u32, u32);

#[derive(Debug, Error)]
pub enum LevelError {
    #[error("Invalid level value")]
    InvalidValue,
}

impl Level {
    pub fn new(integer: u32, decimal: u32) -> Result<Self, LevelError> {
        if integer == 0 || !(0..10).contains(&decimal) {
            return Err(LevelError::InvalidValue);
        }

        Ok(Level(integer, decimal))
    }

    pub fn value(&self) -> f64 {
        self.0 as f64 + self.1 as f64 / 10.0
    }
}

impl TryFrom<(u32, u32)> for Level {
    type Error = LevelError;

    fn try_from(value: (u32, u32)) -> Result<Self, Self::Error> {
        if value.0 == 0 || !(0..10).contains(&value.1) {
            return Err(LevelError::InvalidValue);
        }

        Ok(Level(value.0, value.1))
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.1 < 5 {
            write!(f, "{}", self.0)
        } else {
            write!(f, "{}+", self.0)
        }
    }
}
