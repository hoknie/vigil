pub fn time_of_day(stamp: &str) -> &str {
    match stamp.len() >= 19 && stamp.as_bytes()[10] == b'T' {
        true => &stamp[11..19],
        false => stamp,
    }
}

pub fn utc(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let rest = seconds.rem_euclid(86_400);
    let (year, month, day) = civil(days);

    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        rest / 3_600,
        (rest % 3_600) / 60,
        rest % 60
    )
}

fn civil(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let of_era = shifted.rem_euclid(146_097);
    let year_of_era = (of_era - of_era / 1_460 + of_era / 36_524 - of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let moved = (5 * of_year + 2) / 153;
    let day = of_year - (153 * moved + 2) / 5 + 1;
    let month = match moved < 10 {
        true => moved + 3,
        false => moved - 9,
    };

    (year + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stamp_is_shown_as_the_time_of_day_and_a_stamp_of_another_shape_is_shown_whole() {
        assert_eq!(time_of_day("2026-09-12T20:29:41.000Z"), "20:29:41");
        assert_eq!(time_of_day("a moment ago"), "a moment ago");
    }

    #[test]
    fn a_moment_the_kernel_counts_in_seconds_is_drawn_as_a_day_and_a_clock() {
        assert_eq!(utc(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(utc(1_757_419_200), "2025-09-09 12:00:00 UTC");
        assert_eq!(utc(1_767_225_600), "2026-01-01 00:00:00 UTC");
    }

    #[test]
    fn the_day_a_leap_year_adds_is_counted_rather_than_slid_past() {
        assert_eq!(utc(1_709_164_800), "2024-02-29 00:00:00 UTC");
        assert_eq!(utc(1_709_251_200), "2024-03-01 00:00:00 UTC");
    }
}
