use crate::views::sorted::{first_seen_key, last_run_key, program_key, runs_key};

#[test]
fn a_last_run_key_puts_two_launches_in_the_order_their_moments_put_them() {
    let moments = [
        None,
        Some((0, 0, 0)),
        Some((0, 0, 1)),
        Some((9, 999, 1)),
        Some((10, 0, 0)),
        Some((10, 1000, 0)),
        Some((999_999_999, 5, 12)),
        Some((1_757_419_203, 412, 3421)),
        Some((1_757_419_203, 412, 99)),
        Some((u64::MAX, u64::MAX, u64::MAX)),
    ];

    for left in moments {
        for right in moments {
            assert_eq!(
                last_run_key(left).cmp(&last_run_key(right)),
                left.cmp(&right),
                "{left:?} against {right:?}: the index compares text, and a second written \
                 with nine digits must not sort after one written with ten"
            );
        }
    }
}

#[test]
fn a_run_count_key_puts_two_launches_in_the_order_their_counts_put_them() {
    let counts = [0u64, 1, 2, 9, 10, 11, 99, 100, 109, 1_000, u64::MAX];

    for left in counts {
        for right in counts {
            assert_eq!(
                runs_key(left).cmp(&runs_key(right)),
                left.cmp(&right),
                "{left} against {right}: the index compares text, and a count written with one \
                 digit must not sort after one written with two"
            );
        }
    }
}

const NAMES: &[&str] = &[
    "", "a", "ab", "a\0", "a\0b", "a\0\0", "a\u{1}", "\0", "\u{1}", "id", "id\0x", "nc", "é", "z",
];

const PATHS: &[&str] = &["", "/", "/a", "/usr/bin/a", "\0", "/z\0", "/é"];

#[test]
fn a_program_key_puts_two_programs_in_the_order_their_name_and_then_their_path_put_them() {
    for left_name in NAMES {
        for left_path in PATHS {
            for right_name in NAMES {
                for right_path in PATHS {
                    assert_eq!(
                        program_key(left_name, left_path).cmp(&program_key(right_name, right_path)),
                        (left_name, left_path).cmp(&(right_name, right_path)),
                        "{left_name:?} {left_path:?} against {right_name:?} {right_path:?}: the \
                         index compares one string per column, and a name that is the start of \
                         another name or holds a NUL must not let the path decide before the \
                         name has"
                    );
                }
            }
        }
    }
}

#[test]
fn a_launch_never_seen_goes_before_every_launch_seen_even_one_seen_at_an_empty_moment() {
    let moments = [
        None,
        Some(""),
        Some("+"),
        Some("2026-09-09"),
        Some("2026-09-10"),
    ];

    for left in moments {
        for right in moments {
            assert_eq!(
                first_seen_key(left).cmp(&first_seen_key(right)),
                left.cmp(&right),
                "{left:?} against {right:?}: the list compares the moment as an option, so the \
                 index key has to keep nothing apart from an empty text"
            );
        }
    }
}
