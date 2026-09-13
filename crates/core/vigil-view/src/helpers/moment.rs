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
    fn a_stamp_is_shown_as_the_time_of_day_and_a_stamp_of_another_shape_is_shown_whole() {
        assert_eq!(time_of_day("2026-09-12T20:29:41.000Z"), "20:29:41");
        assert_eq!(time_of_day("a moment ago"), "a moment ago");
    }
}
