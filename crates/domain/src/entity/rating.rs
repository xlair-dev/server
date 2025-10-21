#[derive(Debug, Default)]
pub struct Rating(u32);

impl Rating {
    pub fn new(rating: u32) -> Self {
        Self(rating)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}
