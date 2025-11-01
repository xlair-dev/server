use serde::Serialize;
use usecase::model::ranking::{
    RatingRankingDto, RatingRankingEntryDto, SheetScoreRankingDto, SheetScoreRankingEntryDto,
    TotalScoreRankingDto, TotalScoreRankingEntryDto, XpRankingDto, XpRankingEntryDto,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetScoreRankingEntryResponse {
    pub rank: u32,
    pub user_id: String,
    pub display_name: String,
    pub score: u32,
}

impl From<SheetScoreRankingEntryDto> for SheetScoreRankingEntryResponse {
    fn from(dto: SheetScoreRankingEntryDto) -> Self {
        Self {
            rank: dto.rank,
            user_id: dto.user_id,
            display_name: dto.display_name,
            score: dto.score,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetScoreRankingResponse {
    pub sheet_id: String,
    pub entries: Vec<SheetScoreRankingEntryResponse>,
}

impl From<SheetScoreRankingDto> for SheetScoreRankingResponse {
    fn from(dto: SheetScoreRankingDto) -> Self {
        Self {
            sheet_id: dto.sheet_id,
            entries: dto.entries.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalScoreRankingEntryResponse {
    pub rank: u32,
    pub user_id: String,
    pub display_name: String,
    pub total_score: u64,
}

impl From<TotalScoreRankingEntryDto> for TotalScoreRankingEntryResponse {
    fn from(dto: TotalScoreRankingEntryDto) -> Self {
        Self {
            rank: dto.rank,
            user_id: dto.user_id,
            display_name: dto.display_name,
            total_score: dto.total_score,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalScoreRankingResponse {
    pub entries: Vec<TotalScoreRankingEntryResponse>,
}

impl From<TotalScoreRankingDto> for TotalScoreRankingResponse {
    fn from(dto: TotalScoreRankingDto) -> Self {
        Self {
            entries: dto.entries.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingRankingEntryResponse {
    pub rank: u32,
    pub user_id: String,
    pub display_name: String,
    pub rating: u32,
}

impl From<RatingRankingEntryDto> for RatingRankingEntryResponse {
    fn from(dto: RatingRankingEntryDto) -> Self {
        Self {
            rank: dto.rank,
            user_id: dto.user_id,
            display_name: dto.display_name,
            rating: dto.rating,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingRankingResponse {
    pub entries: Vec<RatingRankingEntryResponse>,
}

impl From<RatingRankingDto> for RatingRankingResponse {
    fn from(dto: RatingRankingDto) -> Self {
        Self {
            entries: dto.entries.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XpRankingEntryResponse {
    pub rank: u32,
    pub user_id: String,
    pub display_name: String,
    pub xp: u32,
}

impl From<XpRankingEntryDto> for XpRankingEntryResponse {
    fn from(dto: XpRankingEntryDto) -> Self {
        Self {
            rank: dto.rank,
            user_id: dto.user_id,
            display_name: dto.display_name,
            xp: dto.xp,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XpRankingResponse {
    pub entries: Vec<XpRankingEntryResponse>,
}

impl From<XpRankingDto> for XpRankingResponse {
    fn from(dto: XpRankingDto) -> Self {
        Self {
            entries: dto.entries.into_iter().map(Into::into).collect(),
        }
    }
}
