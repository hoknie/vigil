pub fn time_of_day(stamp: &str) -> &str {
    match stamp.len() >= 19 && stamp.as_bytes()[10] == b'T' {
        true => &stamp[11..19],
        false => stamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_the_clock_out_of_a_contract_timestamp() {
        assert_eq!(time_of_day("2026-09-09T09:00:00.000Z"), "09:00:00");
    }

    #[test]
    fn something_that_is_not_one_is_shown_rather_than_cut_to_nothing() {
        assert_eq!(time_of_day("never"), "never");
        assert_eq!(time_of_day(""), "");
    }
}
