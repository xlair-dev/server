use std::fmt::{Display, Formatter, Result};

// TODO: Determine the difficulty names
#[derive(Debug)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Difficulty::Easy => write!(f, "EASY"),
            Difficulty::Normal => write!(f, "NORMAL"),
            Difficulty::Hard => write!(f, "HARD"),
        }
    }
}
