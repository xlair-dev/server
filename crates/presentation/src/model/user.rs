use serde::{Deserialize, Serialize};
use usecase::model::user::{UserCreditsDto, UserDataDto, UserRecordDto, UserRegisterDto};

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

#[derive(Deserialize)]
pub struct FindUserQuery {
    pub card: String,
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
    pub rating: u32,
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
        rating: u32,
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

#[derive(Serialize)]
pub struct CreditsIncrementResponse {
    pub credits: u32,
}

impl From<UserCreditsDto> for CreditsIncrementResponse {
    fn from(dto: UserCreditsDto) -> Self {
        Self {
            credits: dto.credits,
        }
    }
}

#[derive(Serialize)]
pub struct UserRecordResponse {
    pub id: String,
    #[serde(rename = "sheetId")]
    pub sheet_id: String,
    pub score: u32,
    #[serde(rename = "clearType")]
    pub clear_type: String,
    #[serde(rename = "playCount")]
    pub play_count: u32,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

impl From<UserRecordDto> for UserRecordResponse {
    fn from(dto: UserRecordDto) -> Self {
        let clear_type = match dto.clear_type {
            domain::entity::clear_type::ClearType::Fail => "failed",
            domain::entity::clear_type::ClearType::Clear => "clear",
            domain::entity::clear_type::ClearType::FullCombo => "fullcombo",
            domain::entity::clear_type::ClearType::AllPerfect => "perfect",
        }
        .to_string();

        Self {
            id: dto.id,
            sheet_id: dto.sheet_id,
            score: dto.score,
            clear_type,
            play_count: dto.play_count,
            updated_at: dto.updated_at.to_string(),
        }
    }
}
