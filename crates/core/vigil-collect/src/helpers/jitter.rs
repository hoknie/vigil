pub fn steadied(held: Option<i64>, read: i64, tolerance: i64) -> i64 {
    match held {
        Some(before) if read.abs_diff(before) <= tolerance.unsigned_abs() => before,
        _ => read,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reading_within_the_tolerance_is_the_reading_before_it_and_not_a_change() {
        let held = Some(1_757_419_200);

        assert_eq!(steadied(held, 1_757_419_199, 2), 1_757_419_200);
        assert_eq!(steadied(held, 1_757_419_201, 2), 1_757_419_200);
        assert_eq!(
            steadied(held, 1_757_419_200, 2),
            1_757_419_200,
            "two whole numbers taken from two clocks disagree by a second on most readings, and \
             a value that moves by a second on most readings is a change on most readings"
        );
    }

    #[test]
    fn a_reading_past_the_tolerance_is_the_new_one_and_the_old_one_is_let_go() {
        let held = Some(1_757_419_200);

        assert_eq!(steadied(held, 1_757_419_500, 2), 1_757_419_500);
        assert_eq!(steadied(held, 1_757_418_900, 2), 1_757_418_900);
    }

    #[test]
    fn the_first_reading_of_all_is_held_whatever_it_says() {
        assert_eq!(steadied(None, 1_757_419_200, 2), 1_757_419_200);
        assert_eq!(steadied(None, 0, 2), 0);
    }

    #[test]
    fn a_drift_of_a_second_at_a_time_is_let_through_one_step_at_a_time_and_never_hidden() {
        let mut held = None;
        let mut published = Vec::new();

        for step in 0..6 {
            let now = steadied(held, 1_757_419_200 + step * 3, 2);
            published.push(now);
            held = Some(now);
        }

        assert_eq!(
            published,
            vec![
                1_757_419_200,
                1_757_419_203,
                1_757_419_206,
                1_757_419_209,
                1_757_419_212,
                1_757_419_215
            ],
            "holding a value cannot turn into losing it: a clock walking away is still a clock \
             walking away, and the rule that reads the difference is the one that decides"
        );
    }
}
