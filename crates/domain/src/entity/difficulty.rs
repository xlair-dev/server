use thiserror::Error;

const DIFFICULTY_NAMES: [&str; 3] = [
    // TODO: Determine the difficulty names
    "EASY", "NORMAL", "HARD",
];

#[derive(Debug)]
pub struct Difficulty(u32);

#[derive(Debug, Error)]
pub enum DifficultyError {
    #[error("Invalid difficulty value")]
    InvalidValue,
}

impl TryFrom<u32> for Difficulty {
    type Error = DifficultyError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if !(0..DIFFICULTY_NAMES.len() as u32).contains(&value) {
            return Err(DifficultyError::InvalidValue);
        }

        Ok(Difficulty(value))
    }
}

impl Difficulty {
    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn name(&self) -> &str {
        DIFFICULTY_NAMES[self.0 as usize]
    }
}
