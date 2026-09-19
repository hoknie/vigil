use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0);

    format(seconds)
}

pub fn format(seconds: u64) -> String {
    let days = seconds / 86_400;
    let of_the_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days as i64);

    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.000Z",
        of_the_day / 3_600,
        (of_the_day % 3_600) / 60,
        of_the_day % 60,
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
    let month = match shifted_month < 10 {
        true => shifted_month + 3,
        false => shifted_month - 9,
    } as u32;

    (year + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_time_in_the_document_is_written_the_way_every_other_time_in_this_product_is() {
        assert_eq!(format(1_789_635_601), "2026-09-17T09:00:01.000Z");
        assert_eq!(format(0), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn the_hour_is_the_same_hour_wherever_this_host_believes_it_is() {
        assert!(
            now().ends_with('Z'),
            "a local time in a file read on another machine during an incident is a time \
             somebody has to convert under pressure"
        );
    }
}
