use std::borrow::Cow;
use std::sync::Arc;

use super::{Assembled, Index, Placed, Rows};
use crate::Facet;
use crate::types::{RowKey, Sorting};

#[test]
fn rows_named_by_their_numbers_read_as_the_rows_they_name() {
    let index = index();
    let assembled = Assembled::Ordered(vec![4, 0, 2]);
    let rows = Rows::of(&index, &assembled);

    assert_eq!(rows.len(), 3);
    assert_eq!(
        (0..rows.len())
            .filter_map(|place| rows.key(place))
            .collect::<Vec<&str>>(),
        vec!["d", "b", "c"],
        "a list the index answered keeps the numbers of its rows, and reads each row from the \
         index when it is asked for"
    );
    assert_eq!(rows.key(1), Some("b"));
    assert!(
        matches!(rows.get(1), Some(Cow::Borrowed(row)) if row.key == "b"),
        "a row of the list is lent from the index and never copied"
    );
    assert!(rows.get(3).is_none());
    assert_eq!(
        Rows::built(&rows.to_vec()).to_vec(),
        rows.to_vec(),
        "a footer is written from either, and must read the same rows from both"
    );
}

#[test]
fn a_tree_of_row_numbers_builds_its_headings_from_the_rows_under_them_and_copies_no_row() {
    let mut index = Index::new(0);
    for (key, heading, name) in [
        ("tcp|:80", "program|/usr/sbin/nginx", Some("nginx")),
        ("tcp|:443", "program|/usr/sbin/nginx", Some("nginx")),
        ("udp|:53", "unresolved", None),
    ] {
        let mut row = RowKey::of(key).under(1).beneath(heading);
        row.named = name.map(Arc::from);
        index.push(row, key, 0, Vec::new());
    }
    let assembled = Assembled::Gathered(vec![
        Placed::Heading {
            first: 0,
            gathers: 2,
            opened: true,
        },
        Placed::Row(0),
        Placed::Row(1),
        Placed::Heading {
            first: 2,
            gathers: 1,
            opened: false,
        },
    ]);
    let rows = Rows::of(&index, &assembled);

    let mut nginx = RowKey::of("program|/usr/sbin/nginx")
        .of_its_own()
        .gathering(2)
        .opened(true);
    nginx.named = Some(Arc::from("nginx"));
    assert_eq!(
        rows.to_vec(),
        vec![
            nginx,
            index.row(0).clone(),
            index.row(1).clone(),
            RowKey::of("unresolved").of_its_own().gathering(1),
        ],
        "a heading is made from the first row under it when it is asked for: its key is the \
         heading that row names, its name is that row's name"
    );
    assert!(
        matches!(rows.get(1), Some(Cow::Borrowed(_))),
        "a row under a heading is lent from the index and never copied"
    );
    assert_eq!(rows.key(3), Some("unresolved"));
}

#[test]
fn every_row_under_one_key_is_found_by_that_key() {
    let mut index = Index::new(0);
    for key in ["b", "a", "b", "c"] {
        index.push(RowKey::of(key), key, 1, Vec::new());
    }

    assert_eq!(
        index.keyed("b"),
        &[0, 2],
        "one unit can stand under two parents, so one key can name two rows"
    );
    assert_eq!(index.keyed("a"), &[1]);
    assert!(index.keyed("z").is_empty());
}

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

pub(super) fn index() -> Index {
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

pub(super) fn keys(index: &Index, at: &[usize]) -> Vec<String> {
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
