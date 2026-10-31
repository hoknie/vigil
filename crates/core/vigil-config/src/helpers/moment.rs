const SECONDS_A_DAY: i64 = 86_400;

pub fn moment(written: &str) -> Result<String, String> {
    let refusal = || {
        format!(
            "until: {written:?} is not a moment this agent can compare with. Write 2026-12-31 \
             or 2026-12-31T00:00:00.000Z"
        )
    };

    let (date, time) = match written.split_once('T') {
        None => (written, None),
        Some((date, time)) => (date, Some(time)),
    };
    let (year, month, day) = civil_date(date).ok_or_else(refusal)?;

    let Some(time) = time else {
        return Ok(format!("{year:04}-{month:02}-{day:02}T00:00:00.000Z"));
    };
    let (clock, offset_seconds) = zoned(time).ok_or_else(refusal)?;
    let (seconds_of_day, milliseconds) = clock_of(clock).ok_or_else(refusal)?;

    let instant =
        days_from_civil(year, month, day) * SECONDS_A_DAY + seconds_of_day - offset_seconds;
    let (year, month, day) = civil_from_days(instant.div_euclid(SECONDS_A_DAY));
    let within = instant.rem_euclid(SECONDS_A_DAY);
    if !(0..=9_999).contains(&year) {
        return Err(refusal());
    }

    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{milliseconds:03}Z",
        within / 3_600,
        within % 3_600 / 60,
        within % 60
    ))
}

fn civil_date(date: &str) -> Option<(i64, u32, u32)> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year: i64 = digits(&date[0..4])?.into();
    let month = digits(&date[5..7])?;
    let day = digits(&date[8..10])?;

    match (1..=12).contains(&month) && (1..=days_in(year, month)).contains(&day) {
        true => Some((year, month, day)),
        false => None,
    }
}

fn zoned(time: &str) -> Option<(&str, i64)> {
    if let Some(clock) = time.strip_suffix('Z') {
        return Some((clock, 0));
    }

    let at = time.len().checked_sub(6)?;
    let (clock, zone) = time.split_at(at);
    let sign = match zone.as_bytes()[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    if zone.as_bytes()[3] != b':' {
        return None;
    }
    let hours = digits(&zone[1..3])?;
    let minutes = digits(&zone[4..6])?;
    if hours > 23 || minutes > 59 {
        return None;
    }

    Some((clock, sign * i64::from(hours * 3_600 + minutes * 60)))
}

fn clock_of(clock: &str) -> Option<(i64, u32)> {
    let (whole, fraction) = match clock.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (clock, None),
    };

    let bytes = whole.as_bytes();
    if bytes.len() != 8 || bytes[2] != b':' || bytes[5] != b':' {
        return None;
    }
    let hours = digits(&whole[0..2])?;
    let minutes = digits(&whole[3..5])?;
    let seconds = digits(&whole[6..8])?;
    if hours > 23 || minutes > 59 || seconds > 59 {
        return None;
    }

    let milliseconds = match fraction {
        None => 0,
        Some(fraction) => {
            if fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            format!("{fraction:0<3}")[..3].parse().ok()?
        }
    };

    Some((
        i64::from(hours * 3_600 + minutes * 60 + seconds),
        milliseconds,
    ))
}

fn digits(text: &str) -> Option<u32> {
    match text.bytes().all(|byte| byte.is_ascii_digit()) {
        true => text.parse().ok(),
        false => None,
    }
}

fn days_in(year: i64, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
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

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    } as u32;
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refused(written: &str) -> String {
        moment(written).expect_err("must not be accepted")
    }

    #[test]
    fn a_date_is_the_start_of_that_day_in_utc() {
        assert_eq!(
            moment("2026-12-31").as_deref(),
            Ok("2026-12-31T00:00:00.000Z")
        );
    }

    #[test]
    fn an_offset_is_turned_into_utc_across_a_day_month_and_year() {
        assert_eq!(
            moment("2026-12-31T03:00:00+03:00").as_deref(),
            Ok("2026-12-31T00:00:00.000Z")
        );
        assert_eq!(
            moment("2027-01-01T01:30:00+03:00").as_deref(),
            Ok("2026-12-31T22:30:00.000Z"),
            "back across a year"
        );
        assert_eq!(
            moment("2026-12-31T22:00:00-05:00").as_deref(),
            Ok("2027-01-01T03:00:00.000Z"),
            "forward across a year"
        );
        assert_eq!(
            moment("2028-03-01T00:30:00+01:00").as_deref(),
            Ok("2028-02-29T23:30:00.000Z"),
            "back into the last day of February of a leap year"
        );
        assert_eq!(
            moment("2026-04-30T23:00:00-02:00").as_deref(),
            Ok("2026-05-01T01:00:00.000Z"),
            "forward across a month"
        );
        assert_eq!(
            moment("2026-12-31T09:30:00Z").as_deref(),
            Ok("2026-12-31T09:30:00.000Z")
        );
    }

    #[test]
    fn a_fraction_of_any_length_is_cut_to_milliseconds() {
        assert_eq!(
            moment("2026-12-31T09:30:00.5Z").as_deref(),
            Ok("2026-12-31T09:30:00.500Z")
        );
        assert_eq!(
            moment("2026-12-31T09:30:00.123456789+00:00").as_deref(),
            Ok("2026-12-31T09:30:00.123Z")
        );
        assert_eq!(
            moment("2026-12-31T09:30:00.000Z").as_deref(),
            Ok("2026-12-31T09:30:00.000Z"),
            "and the agent's own form reads back unchanged"
        );
    }

    #[test]
    fn a_day_that_does_not_exist_is_refused() {
        for written in [
            "2026-02-30",
            "2026-13-01",
            "2026-04-31",
            "2026-12-31T25:00:00Z",
            "2026-12-31T23:60:00Z",
            "2026-12-31T23:59:60Z",
            "2026-12-31T00:00:00+24:00",
        ] {
            assert!(moment(written).is_err(), "{written} was accepted");
        }
    }

    #[test]
    fn february_29th_is_a_day_only_in_a_leap_year() {
        assert!(moment("2028-02-29").is_ok());
        assert!(moment("2000-02-29").is_ok(), "divisible by 400");
        assert!(moment("2026-02-29").is_err());
        assert!(
            moment("2100-02-29").is_err(),
            "divisible by 100 and not 400"
        );
    }

    #[test]
    fn a_form_this_agent_cannot_compare_is_refused_with_how_to_write_it() {
        assert_eq!(
            refused("31.12.2026"),
            "until: \"31.12.2026\" is not a moment this agent can compare with. Write \
             2026-12-31 or 2026-12-31T00:00:00.000Z"
        );
        for written in [
            "2026-12-31 10:00",
            "2026-12-31T10:00",
            "2026-12-31T10:00:00",
            "2026-12-31T10:00:00.Z",
            "2026-12-31T10:00:00+0300",
            "2026-1-31",
            "tomorrow",
            "",
        ] {
            assert!(
                refused(written).contains("is not a moment this agent can compare with"),
                "{written:?}"
            );
        }
    }
}
