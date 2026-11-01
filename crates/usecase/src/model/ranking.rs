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
    pub distribution: ScoreDistributionDto,
}

impl SheetScoreRankingDto {
    pub fn new(
        sheet_id: String,
        entries: Vec<SheetScoreRankingEntryDto>,
        distribution: ScoreDistributionDto,
    ) -> Self {
        Self {
            sheet_id,
            entries,
            distribution,
        }
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
    pub distribution: ScoreDistributionDto,
}

impl TotalScoreRankingDto {
    pub fn new(
        entries: Vec<TotalScoreRankingEntryDto>,
        distribution: ScoreDistributionDto,
    ) -> Self {
        Self {
            entries,
            distribution,
        }
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
    pub distribution: ScoreDistributionDto,
}

impl RatingRankingDto {
    pub fn new(entries: Vec<RatingRankingEntryDto>, distribution: ScoreDistributionDto) -> Self {
        Self {
            entries,
            distribution,
        }
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
    pub distribution: ScoreDistributionDto,
}

impl XpRankingDto {
    pub fn new(entries: Vec<XpRankingEntryDto>, distribution: ScoreDistributionDto) -> Self {
        Self {
            entries,
            distribution,
        }
    }
}
const MAX_DISTRIBUTION_BINS: u64 = 20;

#[derive(Debug, Clone)]
pub struct ScoreDistributionBinDto {
    pub range_start: u64,
    pub range_end: u64,
    pub player_count: u64,
}

#[derive(Debug, Clone)]
pub struct ScoreDistributionDto {
    pub bins: Vec<ScoreDistributionBinDto>,
}

impl ScoreDistributionDto {
    pub fn from_values(values: &[u64]) -> Self {
        let Some(minimum) = values.iter().copied().min() else {
            return Self { bins: Vec::new() };
        };
        let maximum = values.iter().copied().max().unwrap_or(minimum);
        let span = u128::from(maximum) - u128::from(minimum) + 1;
        let bin_count = span.min(u128::from(MAX_DISTRIBUTION_BINS));
        let mut counts = vec![0_u64; bin_count as usize];

        for value in values {
            let offset = u128::from(*value) - u128::from(minimum);
            let index = (offset * bin_count / span).min(bin_count - 1) as usize;
            counts[index] += 1;
        }

        let bins = counts
            .into_iter()
            .enumerate()
            .map(|(index, player_count)| {
                let start_offset = (index as u128 * span).div_ceil(bin_count);
                let end_offset = ((index as u128 + 1) * span).div_ceil(bin_count) - 1;
                ScoreDistributionBinDto {
                    range_start: (u128::from(minimum) + start_offset) as u64,
                    range_end: (u128::from(minimum) + end_offset) as u64,
                    player_count,
                }
            })
            .collect();
        Self { bins }
    }
}

#[cfg(test)]
mod tests {
    use super::ScoreDistributionDto;

    #[test]
    fn empty_values_produce_no_bins() {
        assert!(ScoreDistributionDto::from_values(&[]).bins.is_empty());
    }

    #[test]
    fn equal_values_share_one_bin() {
        let distribution = ScoreDistributionDto::from_values(&[42, 42, 42]);

        assert_eq!(distribution.bins.len(), 1);
        assert_eq!(distribution.bins[0].range_start, 42);
        assert_eq!(distribution.bins[0].range_end, 42);
        assert_eq!(distribution.bins[0].player_count, 3);
    }

    #[test]
    fn large_values_are_binned_without_overflow() {
        let distribution = ScoreDistributionDto::from_values(&[0, u64::MAX]);

        assert_eq!(distribution.bins.len(), 20);
        assert_eq!(distribution.bins.first().unwrap().range_start, 0);
        assert_eq!(distribution.bins.last().unwrap().range_end, u64::MAX);
        assert_eq!(
            distribution
                .bins
                .iter()
                .map(|bin| bin.player_count)
                .sum::<u64>(),
            2
        );
    }
}
