use crate::{
    entity::{level::Level, rating::Rating},
    repository::record::RecordWithMetadata,
};

/// Calculates the average of the three highest non-test sheet ratings.
/// Missing rating slots are treated as zero.
pub fn calculate_user_rating(records: &[RecordWithMetadata]) -> Rating {
    const RATING_SLOTS: u32 = 3;
    let mut values: Vec<u32> = records
        .iter()
        .filter(|entry| !entry.is_test)
        .map(|entry| calculate_sheet_rating(&entry.level, *entry.record.score()))
        .collect();

    values.sort_unstable_by(|a, b| b.cmp(a));
    let total: u32 = values.into_iter().take(RATING_SLOTS as usize).sum();
    Rating::new(total / RATING_SLOTS)
}

fn calculate_sheet_rating(level: &Level, score: u32) -> u32 {
    let (integer, decimal) = level.components();
    let base = integer * 100 + decimal * 10;
    let bonus = compute_score_bonus(score);
    let total = base as i64 + bonus as i64;
    if total < 0 { 0 } else { total as u32 }
}

fn compute_score_bonus(score: u32) -> i32 {
    const ANCHORS: [(u32, i32); 9] = [
        (700_000, -200),
        (750_000, -150),
        (800_000, -100),
        (850_000, -50),
        (900_000, 0),
        (950_000, 50),
        (1_000_000, 100),
        (1_050_000, 150),
        (1_090_000, 200),
    ];

    if score <= ANCHORS[0].0 {
        return ANCHORS[0].1;
    }

    if score >= ANCHORS[ANCHORS.len() - 1].0 {
        return ANCHORS[ANCHORS.len() - 1].1;
    }

    for window in ANCHORS.windows(2) {
        let lower = window[0];
        let upper = window[1];
        if (lower.0..=upper.0).contains(&score) {
            let range = (upper.0 - lower.0) as i64;
            let position = (score - lower.0) as i64;
            let diff = (upper.1 - lower.1) as i64;
            let bonus = lower.1 as i64 + diff * position / range;
            return bonus as i32;
        }
    }

    ANCHORS[0].1
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::{
        entity::{clear_type::ClearType, record::Record},
        repository::record::RecordWithMetadata,
    };

    fn record_with_rating(level: (u32, u32), score: u32, sheet_id: &str) -> RecordWithMetadata {
        let timestamp = NaiveDate::from_ymd_opt(2025, 10, 26)
            .and_then(|date| date.and_hms_opt(12, 0, 0))
            .map(|value| value.and_utc())
            .expect("valid test timestamp");
        let level = Level::new(level.0, level.1).expect("valid test level");
        let record = Record::new(
            format!("record-{sheet_id}"),
            "user-1".to_owned(),
            sheet_id.to_owned(),
            score,
            ClearType::Clear,
            1,
            timestamp,
        );
        RecordWithMetadata::new(record, level, false)
    }

    #[test]
    fn fills_unplayed_rating_slots_with_zero() {
        let records = [record_with_rating((14, 0), 1_000_000, "sheet-1")];

        assert_eq!(calculate_user_rating(&records).value(), 500);
    }

    #[test]
    fn averages_only_the_best_three_records_over_three_slots() {
        let records = [
            record_with_rating((14, 0), 1_000_000, "sheet-1"),
            record_with_rating((13, 0), 1_000_000, "sheet-2"),
            record_with_rating((12, 0), 1_000_000, "sheet-3"),
            record_with_rating((11, 0), 1_000_000, "sheet-4"),
        ];

        assert_eq!(calculate_user_rating(&records).value(), 1400);
    }
}
