const STEP: u64 = 1024;

const BIGGER: &[&str] = &["KB", "MB", "GB", "TB", "PB", "EB"];

pub fn bytes(size: u64) -> String {
    if size < STEP {
        return format!("{size} B");
    }

    let mut scaled = size as f64 / STEP as f64;
    let mut at = 0usize;
    while scaled >= STEP as f64 && at + 1 < BIGGER.len() {
        scaled /= STEP as f64;
        at += 1;
    }

    format!("{scaled:.1} {}", BIGGER[at])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_that_is_not_zero_is_never_drawn_as_zero() {
        for size in [1u64, 7, 512, 1023] {
            let drawn = bytes(size);
            assert_eq!(drawn, format!("{size} B"), "{size}");
            assert!(
                !drawn.starts_with('0'),
                "{size} bytes rounded away to {drawn}, which reads as nothing at all"
            );
        }
        assert_eq!(bytes(0), "0 B");
    }

    #[test]
    fn the_units_go_up_by_the_thousand_and_twenty_four_the_kernel_counts_in() {
        assert_eq!(bytes(1024), "1.0 KB");
        assert_eq!(bytes(155_648), "152.0 KB");
        assert_eq!(bytes(1024 * 1024), "1.0 MB");
        assert_eq!(bytes(3 * 1024 * 1024 + 512 * 1024), "3.5 MB");
        assert_eq!(bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn the_largest_unit_this_knows_holds_whatever_is_bigger_than_it() {
        assert_eq!(bytes(u64::MAX), "16.0 EB");
    }

    #[test]
    fn a_size_stays_narrow_enough_for_a_column_at_eighty_columns() {
        for size in [0u64, 1023, 1024, 155_648, u64::MAX / 3, u64::MAX] {
            assert!(bytes(size).chars().count() <= 8, "{size}: {}", bytes(size));
        }
    }
}
