use chrono::{DateTime, Utc};
use domain::entity::{clear_type::ClearType, record::Record, user::User};

#[derive(Debug)]
pub struct UserRegisterDto {
    pub card: String,
    pub display_name: String,
    pub is_public: bool,
}

impl UserRegisterDto {
    pub fn new(card: String, display_name: String, is_public: bool) -> Self {
        Self { card, display_name, is_public }
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
    pub is_public: bool,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
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
        is_public: bool,
        is_admin: bool,
        created_at: DateTime<Utc>,
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

impl From<User> for UserDataDto {
    fn from(user: User) -> Self {
        Self::new(
            user.id().to_owned(),
            user.card().to_owned(),
            user.display_name().clone(),
            user.rating().value(),
            user.xp().to_owned(),
            user.credits().to_owned(),
            user.is_public().to_owned(),
            user.is_admin().to_owned(),
            user.created_at().to_owned(),
        )
    }
}

#[derive(Debug)]
pub struct UserCreditsDto {
    pub credits: u32,
}

impl UserCreditsDto {
    pub fn new(credits: u32) -> Self {
        Self { credits }
    }
}

#[derive(Debug)]
pub struct UserRecordDto {
    pub id: String,
    pub sheet_id: String,
    pub score: u32,
    pub clear_type: ClearType,
    pub play_count: u32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct UserUpdateDto {
    pub display_name: String,
    pub is_public: bool,
}

impl UserUpdateDto {
    pub fn new(display_name: String, is_public: bool) -> Self {
        Self {
            display_name,
            is_public,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserRecordSubmissionDto {
    pub sheet_id: String,
    pub score: u32,
    pub clear_type: ClearType,
}

impl UserRecordSubmissionDto {
    pub fn new(sheet_id: String, score: u32, clear_type: ClearType) -> Self {
        Self {
            sheet_id,
            score,
            clear_type,
        }
    }
}

impl UserRecordDto {
    pub fn new(
        id: String,
        sheet_id: String,
        score: u32,
        clear_type: ClearType,
        play_count: u32,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            sheet_id,
            score,
            clear_type,
            play_count,
            updated_at,
        }
    }
}

impl From<Record> for UserRecordDto {
    fn from(record: Record) -> Self {
        Self::new(
            record.id().to_owned(),
            record.sheet_id().to_owned(),
            record.score().to_owned(),
            *record.clear_type(),
            record.play_count().to_owned(),
            record.updated_at().to_owned(),
        )
    }
}
