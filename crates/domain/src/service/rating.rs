use crate::{
    entity::{level::Level, rating::Rating},
    repository::record::RecordWithMetadata,
};

pub fn calculate_user_rating(records: &[RecordWithMetadata]) -> Rating {
    let mut values: Vec<u32> = records
        .iter()
        .filter(|entry| !entry.is_test)
        .map(|entry| calculate_single_track_rating(&entry.level, *entry.record.score()))
        .collect();

    if values.is_empty() {
        return Rating::default();
    }

    values.sort_unstable_by(|a, b| b.cmp(a));
    let count = values.len().min(3);
    let total: u32 = values.into_iter().take(count).sum();
    Rating::new(total / count as u32)
}

fn calculate_single_track_rating(level: &Level, score: u32) -> u32 {
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
