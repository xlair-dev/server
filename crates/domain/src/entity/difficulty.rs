use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    Basic,
    Advanced,
    Master,
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Difficulty::Basic => write!(f, "BASIC"),
            Difficulty::Advanced => write!(f, "ADVANCED"),
            Difficulty::Master => write!(f, "MASTER"),
        }
    }
}
