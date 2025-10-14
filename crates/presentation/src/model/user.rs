use serde::{Deserialize, Serialize};
use usecase::model::user::{UserDataDto, UserRegisterDto};

#[derive(Deserialize)]
pub struct RegisterUserRequest {
    pub card: String,
    pub display_name: String,
}

impl RegisterUserRequest {
    pub fn new(card: String, display_name: String) -> Self {
        Self { card, display_name }
    }
}

impl From<RegisterUserRequest> for UserRegisterDto {
    fn from(request: RegisterUserRequest) -> Self {
        UserRegisterDto::new(request.card, request.display_name)
    }
}

#[derive(Serialize)]
pub struct UserDataResponse {
    pub id: String,
    pub card: String,
    pub display_name: String,
    pub rating: f32,
    pub xp: u32,
    pub credits: u32,
    pub is_admin: bool,
    pub created_at: String,
}

#[allow(clippy::too_many_arguments)]
impl UserDataResponse {
    pub fn new(
        id: String,
        card: String,
        display_name: String,
        rating: f32,
        xp: u32,
        credits: u32,
        is_admin: bool,
        created_at: String,
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

impl From<UserDataDto> for UserDataResponse {
    fn from(user_data: UserDataDto) -> Self {
        Self::new(
            user_data.id,
            user_data.card,
            user_data.display_name,
            user_data.rating,
            user_data.xp,
            user_data.credits,
            user_data.is_admin,
            user_data.created_at.to_string(),
        )
    }
}
