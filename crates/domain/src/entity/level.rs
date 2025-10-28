use std::fmt::{Display, Formatter, Result as FmtResult};

use anyhow::Result;
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

    pub fn components(&self) -> (u32, u32) {
        (self.0, self.1)
    }
}

impl TryFrom<(u32, u32)> for Level {
    type Error = LevelError;

    fn try_from(value: (u32, u32)) -> Result<Self, Self::Error> {
        Level::new(value.0, value.1)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_valid_values() {
        let level = Level::new(12, 3).expect("should construct");
        assert!((level.value() - 12.3).abs() < f64::EPSILON);
    }

    #[test]
    fn new_rejects_invalid_pairs() {
        assert!(matches!(Level::new(0, 3), Err(LevelError::InvalidValue)));
        assert!(matches!(Level::new(10, 12), Err(LevelError::InvalidValue)));
    }

    #[test]
    fn display_uses_plus_suffix_for_upper_half() {
        let upper_half = Level::new(14, 7).expect("should construct");
        assert_eq!(upper_half.to_string(), "14+");

        let lower_half = Level::new(14, 2).expect("should construct");
        assert_eq!(lower_half.to_string(), "14");
    }
}
