#[derive(Debug)]
pub struct SheetScoreRankingEntryDto {
    pub rank: u32,
    pub user_id: String,
    pub display_name: String,
    pub score: u32,
}

impl SheetScoreRankingEntryDto {
    pub fn new(rank: u32, user_id: String, display_name: String, score: u32) -> Self {
        Self {
            rank,
            user_id,
            display_name,
            score,
        }
    }
}

#[derive(Debug)]
pub struct SheetScoreRankingDto {
    pub sheet_id: String,
    pub entries: Vec<SheetScoreRankingEntryDto>,
}

impl SheetScoreRankingDto {
    pub fn new(sheet_id: String, entries: Vec<SheetScoreRankingEntryDto>) -> Self {
        Self { sheet_id, entries }
    }
}

#[derive(Debug)]
pub struct TotalScoreRankingEntryDto {
    pub rank: u32,
    pub user_id: String,
    pub display_name: String,
    pub total_score: u64,
}

impl TotalScoreRankingEntryDto {
    pub fn new(rank: u32, user_id: String, display_name: String, total_score: u64) -> Self {
        Self {
            rank,
            user_id,
            display_name,
            total_score,
        }
    }
}

#[derive(Debug)]
pub struct TotalScoreRankingDto {
    pub entries: Vec<TotalScoreRankingEntryDto>,
}

impl TotalScoreRankingDto {
    pub fn new(entries: Vec<TotalScoreRankingEntryDto>) -> Self {
        Self { entries }
    }
}

#[derive(Debug)]
pub struct RatingRankingEntryDto {
    pub rank: u32,
    pub user_id: String,
    pub display_name: String,
    pub rating: u32,
}

impl RatingRankingEntryDto {
    pub fn new(rank: u32, user_id: String, display_name: String, rating: u32) -> Self {
        Self {
            rank,
            user_id,
            display_name,
            rating,
        }
    }
}

#[derive(Debug)]
pub struct RatingRankingDto {
    pub entries: Vec<RatingRankingEntryDto>,
}

impl RatingRankingDto {
    pub fn new(entries: Vec<RatingRankingEntryDto>) -> Self {
        Self { entries }
    }
}

#[derive(Debug)]
pub struct XpRankingEntryDto {
    pub rank: u32,
    pub user_id: String,
    pub display_name: String,
    pub xp: u32,
}

impl XpRankingEntryDto {
    pub fn new(rank: u32, user_id: String, display_name: String, xp: u32) -> Self {
        Self {
            rank,
            user_id,
            display_name,
            xp,
        }
    }
}

#[derive(Debug)]
pub struct XpRankingDto {
    pub entries: Vec<XpRankingEntryDto>,
}

impl XpRankingDto {
    pub fn new(entries: Vec<XpRankingEntryDto>) -> Self {
        Self { entries }
    }
}
