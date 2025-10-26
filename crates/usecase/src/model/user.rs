use chrono::NaiveDateTime;
use domain::entity::user::User;

#[derive(Debug)]
pub struct UserRegisterDto {
    pub card: String,
    pub display_name: String,
}

impl UserRegisterDto {
    pub fn new(card: String, display_name: String) -> Self {
        Self { card, display_name }
    }
}

#[derive(Debug)]
pub struct UserDataDto {
    pub id: String,
    pub card: String,
    pub display_name: String,
    pub rating: u32,
    pub xp: u32,
    pub credits: u32,
    pub is_admin: bool,
    pub created_at: NaiveDateTime,
}

#[allow(clippy::too_many_arguments)]
impl UserDataDto {
    pub fn new(
        id: String,
        card: String,
        display_name: String,
        rating: u32,
        xp: u32,
        credits: u32,
        is_admin: bool,
        created_at: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            card,
            display_name,
            rating,
            xp,
            credits,
            is_admin,
            created_at,
        }
    }
}

impl From<User> for UserDataDto {
    fn from(user: User) -> Self {
        Self::new(
            user.id().to_owned(),
            user.card().to_owned(),
            user.display_name().clone(),
            user.rating().value(),
            user.xp().to_owned(),
            user.credits().to_owned(),
            user.is_admin().to_owned(),
            user.created_at().to_owned(),
        )
    }
}
