use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Copy)]
pub enum Genre {
    ORIGINAL,
    EXTERNAL,
    OTHER,
}

impl Display for Genre {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Genre::ORIGINAL => write!(f, "ORIGINAL"),
            Genre::EXTERNAL => write!(f, "EXTERNAL"),
            Genre::OTHER => write!(f, "OTHER"),
        }
    }
}
