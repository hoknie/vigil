use std::time::{SystemTime, UNIX_EPOCH};

use vigil_model::Rfc3339;

pub fn now() -> Rfc3339 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or(0);
    format(millis)
}

pub fn ahead(wait: std::time::Duration) -> Rfc3339 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or(0);
    format(now.saturating_add(wait.as_millis() as u64))
}

pub fn days_ago(days: u32) -> Rfc3339 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or(0);
    format(now.saturating_sub(u64::from(days) * 86_400_000))
}

pub fn format(millis: u64) -> Rfc3339 {
    let seconds = millis / 1_000;
    let milliseconds = millis % 1_000;

    let days = seconds / 86_400;
    let seconds_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days as i64);

    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{milliseconds:03}Z",
        seconds_of_day / 3_600,
        (seconds_of_day % 3_600) / 60,
        seconds_of_day % 60,
    )
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = (z - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    } as u32;

    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_the_epoch_and_a_known_moment() {
        assert_eq!(format(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(format(1_788_869_042_104), "2026-09-08T12:04:02.104Z");
    }

    #[test]
    fn gets_the_leap_day_right_from_both_sides() {
        assert_eq!(format(1_709_164_800_000), "2024-02-29T00:00:00.000Z");
        assert_eq!(format(1_709_251_199_999), "2024-02-29T23:59:59.999Z");
        assert_eq!(format(1_709_251_200_000), "2024-03-01T00:00:00.000Z");
    }

    #[test]
    fn a_century_that_is_not_a_leap_year_does_not_shift_the_calendar() {
        assert_eq!(format(951_782_400_000), "2000-02-29T00:00:00.000Z");
        assert_eq!(format(4_102_444_800_000), "2100-01-01T00:00:00.000Z");
    }

    #[test]
    fn a_window_measured_backwards_lands_before_now_and_keeps_the_shape() {
        let now = now();
        let cutoff = days_ago(90);

        assert_eq!(cutoff.len(), now.len());
        assert!(cutoff < now, "{cutoff} is not before {now}");
        assert!(
            days_ago(90) < days_ago(30),
            "a longer window starts earlier"
        );
    }

    #[test]
    fn a_moment_still_to_come_is_written_in_the_same_shape_as_one_that_passed() {
        let now = now();
        let soon = ahead(std::time::Duration::from_secs(30));

        assert_eq!(soon.len(), now.len());
        assert!(soon > now, "{soon} is not after {now}");
        assert!(ahead(std::time::Duration::ZERO) >= now);
    }

    #[test]
    fn the_shape_is_the_one_the_contract_asks_for() {
        let stamp = now();

        assert_eq!(stamp.len(), 24, "{stamp}");
        assert!(stamp.ends_with('Z') && stamp.contains('T'), "{stamp}");
        assert!(
            stamp.starts_with("20"),
            "{stamp}: the clock is not plausible"
        );
    }
}
