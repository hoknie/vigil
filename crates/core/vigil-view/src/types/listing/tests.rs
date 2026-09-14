use super::Index;
use crate::Facet;
use crate::types::{RowKey, Sorting};

fn faceted() -> Index {
    let mut index = Index::new(1);
    for (key, haystack, facets) in [
        ("m", "the reading is not complete", vec![]),
        (
            "a",
            "root /usr/bin/id",
            vec![("user", "root"), ("program", "/usr/bin/id")],
        ),
        (
            "b",
            "alice /usr/bin/id",
            vec![("user", "alice"), ("program", "/usr/bin/id")],
        ),
        (
            "c",
            "root /usr/bin/nc",
            vec![("user", "root"), ("program", "/usr/bin/nc")],
        ),
    ] {
        index.push(RowKey::of(key), haystack, 1, vec![key.to_string()]);
        index.faceted(
            facets
                .into_iter()
                .map(|(name, value)| Facet::new(name, value))
                .collect(),
        );
    }
    index
}

#[test]
fn a_facet_narrows_to_the_rows_that_recorded_it_and_two_facets_to_the_rows_that_recorded_both() {
    let index = faceted();

    for (only, expected) in [
        (vec![], None),
        (vec![("user", "root")], Some(vec!["a", "c"])),
        (vec![("program", "/usr/bin/id")], Some(vec!["a", "b"])),
        (
            vec![("user", "root"), ("program", "/usr/bin/id")],
            Some(vec!["a"]),
        ),
        (
            vec![("user", "root"), ("user", "alice")],
            Some(vec!["a", "c"]),
        ),
        (vec![("user", "nobody")], Some(vec![])),
    ] {
        let only: Vec<Facet> = only
            .into_iter()
            .map(|(name, value)| Facet::new(name, value))
            .collect();

        assert_eq!(
            index.narrowed(&only).map(|at| keys(&index, &at)),
            expected.map(|keys| keys.into_iter().map(String::from).collect::<Vec<String>>()),
            "{only:?}: a row that recorded no facet is never narrowed to, nothing chosen narrows \
             nothing, and of two values of one name the first is chosen, as a view reads it"
        );
    }
}

fn index() -> Index {
    let mut index = Index::new(2);
    for (key, haystack, group, by_name, by_size) in [
        ("b", "nginx /usr/sbin/nginx root", 1, "nginx", "0003"),
        ("a", "sshd /usr/sbin/sshd root", 1, "sshd", "0001"),
        ("c", "Postgres /usr/lib/postgres", 1, "postgres", "0003"),
        ("m", "the reading could not see two sockets", 0, "", ""),
        ("d", "nc /tmp/.x/nc www-data", 1, "nc", "0002"),
    ] {
        index.push(
            RowKey::of(key),
            haystack,
            group,
            vec![by_name.to_string(), by_size.to_string()],
        );
    }
    index
}

fn keys(index: &Index, at: &[usize]) -> Vec<String> {
    at.iter().map(|at| index.row(*at).key.clone()).collect()
}

#[test]
fn an_index_that_recorded_no_facet_narrows_nothing_because_its_list_ignores_them() {
    let only = vec![Facet::new("user", "root")];

    assert_eq!(
        index().narrowed(&only),
        None,
        "a list without facets shows every row whatever facet is kept for it, so its index must \
         not narrow them away"
    );
}

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
            "{search:?}: the rows are looked up by their rarest letter and only those are read, \
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
fn a_column_orders_the_rows_it_was_given_and_keeps_the_rows_about_the_reading_first() {
    let index = index();
    let every = index.found("", None);

    assert_eq!(
        keys(&index, &index.ordered(every.clone(), Sorting::default())),
        vec!["b", "a", "c", "m", "d"]
    );
    assert_eq!(
        keys(
            &index,
            &index.ordered(
                every.clone(),
                Sorting {
                    by: 2,
                    descending: false
                }
            )
        ),
        vec!["m", "a", "d", "b", "c"],
        "the group comes before the column, and two rows equal in the column keep the order \
         they were read in"
    );
    assert_eq!(
        keys(
            &index,
            &index.ordered(
                every,
                Sorting {
                    by: 2,
                    descending: true
                }
            )
        ),
        vec!["m", "b", "c", "d", "a"],
        "descending turns the column round and not the group, and ties still keep the order \
         they were read in"
    );
}

#[test]
fn a_column_the_list_does_not_have_leaves_the_rows_as_they_were_read() {
    let index = index();
    let found = index.found("root", None);

    assert_eq!(
        index.ordered(
            found.clone(),
            Sorting {
                by: 9,
                descending: true
            }
        ),
        found
    );
}
