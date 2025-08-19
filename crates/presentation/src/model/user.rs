use serde::{Deserialize, Serialize};
use usecase::user::dto::{UserDataDto, UserRegisterDto};

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
    pub access_code: String,
    pub card: String,
    pub auth_id: Option<String>,
    pub user_name: Option<String>,
    pub display_name: String,
    pub rating: f32,
    pub xp: u32,
    pub credits: u32,
    pub is_admin: bool,
    pub created_at: String,
}

impl UserDataResponse {
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
        created_at: String,
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

impl From<UserDataDto> for UserDataResponse {
    fn from(user_data: UserDataDto) -> Self {
        Self::new(
            user_data.id,
            user_data.access_code,
            user_data.card,
            user_data.auth_id,
            user_data.user_name,
            user_data.display_name,
            user_data.rating,
            user_data.xp,
            user_data.credits,
            user_data.is_admin,
            user_data.created_at.to_string(),
        )
    }
}
