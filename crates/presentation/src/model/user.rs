use domain::entity::clear_type::ClearType;
use serde::{Deserialize, Serialize};
use usecase::model::user::{
    UserCreditsDto, UserDataDto, UserRecordDto, UserRecordSubmissionDto, UserRegisterDto,
    UserUpdateDto,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterUserRequest {
    pub card: String,
    pub display_name: String,
    pub is_public: bool,
}

impl RegisterUserRequest {
    pub fn new(card: String, display_name: String, is_public: bool) -> Self {
        Self { card, display_name, is_public }
    }
}

#[derive(Deserialize)]
pub struct FindUserQuery {
    pub card: String,
}

impl From<RegisterUserRequest> for UserRegisterDto {
    fn from(request: RegisterUserRequest) -> Self {
        UserRegisterDto::new(request.card, request.display_name, request.is_public)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDataResponse {
    pub id: String,
    pub card: String,
    pub display_name: String,
    pub rating: u32,
    pub xp: u32,
    pub credits: u32,
    pub is_public: bool,
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
        is_public: bool,
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
            is_public,
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
            user_data.is_public,
            user_data.is_admin,
            user_data.created_at.to_rfc3339(),
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
#[serde(rename_all = "camelCase")]
pub struct UserRecordResponse {
    pub id: String,
    pub sheet_id: String,
    pub score: u32,
    pub clear_type: String,
    pub play_count: u32,
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
            updated_at: dto.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRecordRequest {
    pub user_id: String,
    pub sheet_id: String,
    pub score: u32,
    pub clear_type: String,
}

impl TryFrom<UserRecordRequest> for UserRecordSubmissionDto {
    type Error = String;

    fn try_from(request: UserRecordRequest) -> Result<Self, Self::Error> {
        let clear_type = match request.clear_type.as_str() {
            "failed" => ClearType::Fail,
            "clear" => ClearType::Clear,
            "fullcombo" => ClearType::FullCombo,
            "perfect" => ClearType::AllPerfect,
            other => return Err(format!("Unsupported clear type: {other}")),
        };

        Ok(UserRecordSubmissionDto::new(
            request.sheet_id,
            request.score,
            clear_type,
        ))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub display_name: String,
    pub is_public: bool,
}

impl From<UpdateUserRequest> for UserUpdateDto {
    fn from(req: UpdateUserRequest) -> Self {
        UserUpdateDto::new(req.display_name, req.is_public)
    }
}
