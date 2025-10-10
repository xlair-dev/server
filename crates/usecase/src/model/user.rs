use chrono::NaiveDateTime;
use domain::entity::user::User;

pub struct UserRegisterDto {
    pub card: String,
    pub display_name: String,
}

impl UserRegisterDto {
    pub fn new(card: String, display_name: String) -> Self {
        Self { card, display_name }
    }
}

pub struct UserDataDto {
    pub id: String,
    pub access_code: String,
    pub card: String,
    pub auth_id: Option<String>,
    pub user_name: Option<String>,
    pub display_name: String,
    pub rating: f32,
    pub xp: u32,
    pub credits: u32,
    pub is_admin: bool,
    pub created_at: NaiveDateTime,
}

#[allow(clippy::too_many_arguments)]
impl UserDataDto {
    pub fn new(
        id: String,
        access_code: String,
        card: String,
        auth_id: Option<String>,
        user_name: Option<String>,
        display_name: String,
        rating: f32,
        xp: u32,
        credits: u32,
        is_admin: bool,
        created_at: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            access_code,
            card,
            auth_id,
            user_name,
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
            user.access_code().to_owned(),
            user.card().to_owned(),
            user.auth_id().clone(),
            user.user_name().clone(),
            user.display_name().clone(),
            user.rating().value() as f32,
            user.xp().to_owned(),
            user.credits().to_owned(),
            user.is_admin().to_owned(),
            user.created_at().to_owned(),
        )
    }
}
