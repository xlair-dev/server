use std::fmt::Display;
use thiserror::Error;

#[derive(Debug)]
pub struct Level(u32, u32);

#[derive(Debug, Error)]
pub enum LevelError {
    #[error("Invalid level value")]
    InvalidValue,
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

impl Level {
    pub fn value_raw(&self) -> (u32, u32) {
        (self.0, self.1)
    }

    pub fn value(&self) -> String {
        if self.1 < 5 {
            format!("{}", self.0)
        } else {
            format!("{}+", self.0)
        }
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}
