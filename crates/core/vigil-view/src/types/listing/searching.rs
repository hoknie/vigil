use super::tests::{index, keys};

#[test]
fn a_search_finds_exactly_the_rows_whose_text_holds_it_whatever_the_case() {
    let index = index();

    for (search, expected) in [
        ("", vec!["b", "a", "c", "m", "d"]),
        ("usr/sbin", vec!["b", "a"]),
        ("POSTGRES", vec!["c"]),
        ("x/nc", vec!["d"]),
        ("zz", vec![]),
        ("ngin x", vec![]),
        ("rootsshd", vec![]),
    ] {
        let walked: Vec<usize> = (0..index.len())
            .filter(|at| search.is_empty() || index.haystack(*at).contains(&search.to_lowercase()))
            .collect();

        assert_eq!(
            keys(&index, &index.found(search, None)),
            expected,
            "{search:?}"
        );
        assert_eq!(
            index.found(search, None),
            walked,
            "{search:?}: the text of every row is searched at once and never across two rows, \
             and that must find what reading every row finds"
        );
    }
}

#[test]
fn a_longer_search_is_answered_from_what_the_shorter_one_found() {
    let index = index();
    let shorter = index.found("s", None);

    assert_eq!(
        index.found("sshd", Some(&shorter)),
        index.found("sshd", None),
        "each letter typed narrows the rows the last one found instead of starting again"
    );
}

#[test]
fn a_single_character_finds_what_a_walk_over_every_row_finds_the_first_time_and_every_time_after() {
    let index = index();

    for character in ["n", "N", "s", "/", "0", ".", "z", " ", "\u{2603}"] {
        let lowered = character.to_lowercase();
        let walked: Vec<usize> = (0..index.len())
            .filter(|at| index.haystack(*at).contains(&lowered))
            .collect();

        assert_eq!(
            index.found(character, None),
            walked,
            "{character:?}: the first time a character is typed its rows are read from the text"
        );
        assert_eq!(
            index.found(character, None),
            walked,
            "{character:?}: every later time the rows kept for that character answer, and they \
             are the same rows"
        );
    }
}

#[test]
fn a_character_typed_once_is_kept_with_the_index_and_nothing_else_is() {
    let index = index();

    assert!(
        !index.keeps('n'),
        "building the index reads no character of any row, so nothing is kept before a reader \
         types"
    );
    let _ = index.found("N", None);
    assert!(
        index.keeps('n'),
        "a character typed once is kept for the next time it is typed, whatever its case"
    );
    assert!(!index.keeps('s'), "a character nobody typed costs nothing");
    let _ = index.found("ns", None);
    assert!(
        !index.keeps('s'),
        "a longer search reads the rows of a character already kept and keeps no other"
    );
}

#[test]
fn a_longer_search_finds_what_a_walk_over_every_row_finds_whatever_is_kept_before_it() {
    for kept_first in ["", "s", "n", "/"] {
        let index = index();
        if !kept_first.is_empty() {
            let _ = index.found(kept_first, None);
        }
        for search in [
            "nginx",
            "SBIN/",
            "/usr/",
            "sshd root",
            "x/nc",
            "\u{2603}\u{2603}",
            "a\u{2603}",
            "zz",
        ] {
            let lowered = search.to_lowercase();
            let walked: Vec<usize> = (0..index.len())
                .filter(|at| index.haystack(*at).contains(&lowered))
                .collect();

            assert_eq!(
                index.found(search, None),
                walked,
                "{search:?} after {kept_first:?}: a longer search reads only the rows of one kept \
                 character, and every row holding the search holds that character"
            );
        }
    }
}

#[test]
fn a_longer_search_with_nothing_kept_keeps_its_first_character_and_no_other() {
    let index = index();

    let _ = index.found("ngi", None);

    assert!(
        index.keeps('n'),
        "a longer search typed before any character keeps the rows of its first character, and \
         the next search starting with it reads no text"
    );
    assert!(
        !index.keeps('g') && !index.keeps('i'),
        "the other characters of the search are not read out of the whole text"
    );
}
