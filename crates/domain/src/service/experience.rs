/// Calculates the XP granted for a single record submission.
/// Formula: max(1, (score - 900000) / 1000).
pub fn xp_for_score(score: u32) -> u32 {
    let base = score.saturating_sub(900_000);
    let bonus = base / 1_000;
    bonus.max(1)
}

/// Aggregates XP for multiple submissions.
pub fn total_xp<I>(scores: I) -> u32
where
    I: IntoIterator<Item = u32>,
{
    scores
        .into_iter()
        .fold(0u32, |acc, score| acc.saturating_add(xp_for_score(score)))
}
