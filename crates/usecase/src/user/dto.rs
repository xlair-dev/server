pub struct RegisterUserDto {
    pub card: String,
    pub display_name: String,
}

impl RegisterUserDto {
    pub fn new(card: String, display_name: String) -> Self {
        Self { card, display_name }
    }
}
