use std::fmt::{Display, Formatter, Result};

// TODO: Add the rest of the genres
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
