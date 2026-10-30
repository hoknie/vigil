const SECONDS_A_DAY: i64 = 86_400;

pub fn epoch_seconds(written: &str) -> Option<i64> {
    let (date, time) = written.strip_suffix('Z')?.split_once('T')?;

    let mut day = date.splitn(3, '-');
    let year: i64 = day.next()?.parse().ok()?;
    let month: u32 = day.next()?.parse().ok()?;
    let day: u32 = day.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let clock = time.split_once('.').map_or(time, |(clock, _)| clock);
    let mut clock = clock.splitn(3, ':');
    let hours: i64 = clock.next()?.parse().ok()?;
    let minutes: i64 = clock.next()?.parse().ok()?;
    let seconds: i64 = clock.next()?.parse().ok()?;
    if !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) || !(0..=60).contains(&seconds) {
        return None;
    }

    Some(days_from_civil(year, month, day) * SECONDS_A_DAY + hours * 3_600 + minutes * 60 + seconds)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let shifted_month = i64::from((month + 9) % 12);
    let day_of_year = (153 * shifted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_moments_written_the_way_the_dump_writes_them_are_seconds_apart() {
        let first = epoch_seconds("2026-09-17T09:00:00.000Z").expect("reads");
        let second = epoch_seconds("2026-09-17T09:10:00.000Z").expect("reads");

        assert_eq!(second - first, 600);
        assert_eq!(epoch_seconds("1970-01-01T00:00:00Z"), Some(0));
    }

    #[test]
    fn a_moment_that_is_not_one_is_no_moment_rather_than_the_epoch() {
        assert_eq!(epoch_seconds("yesterday"), None);
        assert_eq!(epoch_seconds("2026-13-01T00:00:00.000Z"), None);
        assert_eq!(epoch_seconds("2026-09-17T09:00:00.000+03:00"), None);
    }
}
