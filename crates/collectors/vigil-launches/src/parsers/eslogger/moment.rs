const SECONDS_A_DAY: u64 = 86_400;

pub(super) fn epoch_of(written: &str) -> Option<(u64, u32)> {
    let (date, time) = written.strip_suffix('Z')?.split_once('T')?;

    let mut day = date.splitn(3, '-');
    let year: i64 = day.next()?.parse().ok()?;
    let month: u32 = day.next()?.parse().ok()?;
    let day: u32 = day.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let (clock, fraction) = match time.split_once('.') {
        Some((clock, fraction)) => (clock, fraction),
        None => (time, ""),
    };
    let mut clock = clock.splitn(3, ':');
    let hours: u64 = clock.next()?.parse().ok()?;
    let minutes: u64 = clock.next()?.parse().ok()?;
    let seconds: u64 = clock.next()?.parse().ok()?;
    if hours > 23 || minutes > 59 || seconds > 60 {
        return None;
    }

    let milliseconds: u32 = match fraction.is_empty() {
        true => 0,
        false => {
            if !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            format!("{fraction:0<3}")[..3].parse().ok()?
        }
    };

    let days = u64::try_from(days_from_civil(year, month, day)).ok()?;
    Some((
        days * SECONDS_A_DAY + hours * 3_600 + minutes * 60 + seconds,
        milliseconds,
    ))
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
