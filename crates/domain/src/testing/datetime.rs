use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

/// Returns a reproducible timestamp for fixtures. Implicitly depends on `chrono` being available in
/// the consuming crate.
pub fn sample_timestamp() -> NaiveDateTime {
    timestamp(2025, 10, 21, 12, 0, 0)
}

/// Returns a timestamp slightly ahead of [`sample_timestamp`] for scenarios needing variation.
pub fn later_timestamp() -> NaiveDateTime {
    timestamp(2025, 10, 21, 12, 30, 0)
}

/// Constructs a `NaiveDateTime` from the provided components, panicking if they form an invalid
/// combination.
pub fn timestamp(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> NaiveDateTime {
    let date = NaiveDate::from_ymd_opt(year, month, day).expect("invalid date for fixture");
    let time = NaiveTime::from_hms_opt(hour, minute, second).expect("invalid time for fixture");
    NaiveDateTime::new(date, time)
}
