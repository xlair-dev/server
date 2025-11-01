#[derive(Debug)]
pub struct GlobalStatisticsDto {
    pub total_credits: u64,
    pub total_users: u64,
    pub total_score: u64,
}

impl GlobalStatisticsDto {
    pub fn new(total_credits: u64, total_users: u64, total_score: u64) -> Self {
        Self {
            total_credits,
            total_users,
            total_score,
        }
    }
}
