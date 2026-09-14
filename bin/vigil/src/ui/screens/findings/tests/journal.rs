use super::harness::{drawn, storing};
use crate::ui::{Filter, fixture};

#[test]
fn an_agent_that_says_nothing_about_its_history_says_that_rather_than_naming_a_journal() {
    let mut view = fixture::view();
    view.found.findings.clear();

    let page = drawn(&view, &Filter::default(), 80);

    assert!(page.contains("nothing is filtered out"), "{page}");
    assert!(page.contains("does not say"), "{page}");
    assert!(
        !page.contains("journal holds"),
        "a journal nobody reported is not a journal that is empty: {page}"
    );
}

#[test]
fn a_journal_nothing_has_been_written_to_is_not_a_journal_with_nothing_open_in_it() {
    let fresh = drawn(
        &storing(vigil_model::StoreStatus {
            records: vigil_model::Counted {
                held: 0,
                ceiling: 10_000,
            },
            journal_path: Some("/var/lib/vigil/findings/journal.ndjson".into()),
            ..vigil_model::StoreStatus::default()
        }),
        &Filter::default(),
        80,
    );
    let closed = drawn(
        &storing(vigil_model::StoreStatus {
            records: vigil_model::Counted {
                held: 1_284,
                ceiling: 10_000,
            },
            journal_path: Some("/var/lib/vigil/findings/journal.ndjson".into()),
            ..vigil_model::StoreStatus::default()
        }),
        &Filter::default(),
        80,
    );

    assert!(fresh.contains("No finding has been written"), "{fresh}");
    assert!(!fresh.contains("1284"), "{fresh}");
    assert!(closed.contains("1284 record(s)"), "{closed}");
    assert!(closed.contains("none of them is open"), "{closed}");
    for page in [&fresh, &closed] {
        assert!(
            page.contains("journal.ndjson"),
            "an operator during an incident needs the place, not the number: {page}"
        );
    }
}

#[test]
fn lines_of_the_journal_that_could_not_be_read_are_their_own_sentence() {
    let page = drawn(
        &storing(vigil_model::StoreStatus {
            records: vigil_model::Counted {
                held: 12,
                ceiling: 10_000,
            },
            damaged: 3,
            ..vigil_model::StoreStatus::default()
        }),
        &Filter::default(),
        80,
    );

    assert!(page.contains("3 line(s) of it could not be read"), "{page}");
    assert!(page.contains("counted, not thrown away"), "{page}");
}

#[test]
fn what_the_journal_holds_and_this_screen_does_not_is_named_as_kept_rather_than_lost() {
    let mut view = fixture::view();
    view.found.dropped = 12;
    if let Some(status) = view.status.as_mut() {
        status.agent.store = Some(fixture::store());
    }

    let page = drawn(&view, &Filter::default(), 200);

    assert!(page.contains("12 more in the journal"), "{page}");
    assert!(
        page.contains("/var/lib/vigil/findings/journal.ndjson"),
        "a number without the place to read it is a number nobody can act on: {page}"
    );
    assert!(
        !page.contains("12 dropped"),
        "nothing was destroyed, and the word said it was: {page}"
    );
}
