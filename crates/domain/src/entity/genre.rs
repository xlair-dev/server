use anyhow::{anyhow, Error};

const GENRE_NAMES: [&str; 1] = [
    "ORIGINAL",
    // TODO: Add the rest of the genres
];

#[derive(Debug)]
pub struct Genre(u32);

impl TryFrom<u32> for Genre {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if !(0..GENRE_NAMES.len() as u32).contains(&value) {
            return Err(anyhow!("Invalid genre value"));
        }

        Ok(Genre(value))
    }
}

impl Genre {
    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn name(&self) -> &str {
        GENRE_NAMES[self.0 as usize]
    }
}
