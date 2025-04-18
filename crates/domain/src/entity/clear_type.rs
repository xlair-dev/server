use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum ClearType {
    Fail,
    Clear,
    FullCombo,
    AllPerfect,
}

impl Display for ClearType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ClearType::Fail => write!(f, "FAIL"),
            ClearType::Clear => write!(f, "CLEAR"),
            ClearType::FullCombo => write!(f, "FULL COMBO"),
            ClearType::AllPerfect => write!(f, "ALL PERFECT"),
        }
    }
}
