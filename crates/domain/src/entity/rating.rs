#[derive(Debug)]
pub struct Rating(f64);

impl Rating {
    pub fn new(rating: f64) -> Self {
        Self(rating)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}
