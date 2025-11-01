use serde::Serialize;
use usecase::model::statistics::GlobalStatisticsDto;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalStatisticsResponse {
    pub total_credits: u64,
    pub total_users: u64,
    pub total_score: u64,
}

impl From<GlobalStatisticsDto> for GlobalStatisticsResponse {
    fn from(dto: GlobalStatisticsDto) -> Self {
        Self {
            total_credits: dto.total_credits,
            total_users: dto.total_users,
            total_score: dto.total_score,
        }
    }
}
