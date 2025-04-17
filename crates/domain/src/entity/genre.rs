use std::fmt::{Display, Formatter, Result};

// TODO: Add tests for the Genre struct
#[derive(Debug)]
pub enum Genre {
    ORIGINAL,
}

impl Display for Genre {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Genre::ORIGINAL => write!(f, "ORIGINAL"),
        }
    }
}
