use super::harness::{drawn, drawn_from, row, with_a_store};
use crate::ui::fixture;

#[test]
fn an_agent_that_does_not_report_its_store_says_so_instead_of_drawing_an_empty_one() {
    let page = drawn(80);

    assert!(page.contains("WHAT IS KEPT ON DISK"), "{page}");
    assert!(
        page.contains("does not report what its local history holds"),
        "{page}"
    );
    assert!(
        !page.contains("0 of 0"),
        "an agent that says nothing about its store must not be drawn as an empty store: {page}"
    );
}

#[test]
fn a_store_that_reports_its_numbers_names_the_file_an_operator_would_run_jq_over() {
    let page = drawn_from(&with_a_store(), 120);

    assert!(page.contains("1284 of 10000"), "{page}");
    assert!(page.contains("2.1 MB"), "{page}");
    assert!(
        page.contains("/var/lib/vigil/findings/journal.ndjson"),
        "the operator needs the place to run jq, and looks for it during an incident: {page}"
    );
    assert!(page.contains("2026-08-27T04:11:53.000Z"), "{page}");
}

#[test]
fn each_of_the_four_numbers_called_dropped_says_which_ceiling_it_is_about() {
    let page = drawn_from(&with_a_store(), 200);

    assert!(page.contains("41 past the retention window"), "{page}");
    assert!(page.contains("0 at the ceiling"), "{page}");
    assert!(
        page.contains("on the findings screen"),
        "the ring in the daemon is a fourth ceiling and has to name itself: {page}"
    );
    assert!(
        page.contains("0 of 500"),
        "what is waiting for a receiver is a fifth ceiling and is never a number on its own: \
         {page}"
    );
}

#[test]
fn what_is_waiting_for_a_receiver_is_told_apart_from_an_agent_that_does_not_say() {
    let mut behind = with_a_store();
    if let Some(status) = behind.status.as_mut() {
        status.agent.buffers = Some(vec![fixture::behind("ndjson")]);
    }
    let mut silent = with_a_store();
    if let Some(status) = silent.status.as_mut() {
        status.agent.buffers = None;
    }
    let mut losing = with_a_store();
    if let Some(status) = losing.status.as_mut() {
        status.agent.buffers = Some(vec![fixture::losing("ndjson")]);
    }

    let behind = drawn_from(&behind, 200);
    let silent = drawn_from(&silent, 200);
    let losing = drawn_from(&losing, 200);

    assert!(behind.contains("12 of 500"), "{behind}");
    assert!(
        behind.contains("caught up on the next delivery"),
        "a receiver that is behind has lost nothing, and the screen has to say which of the \
         two this is: {behind}"
    );
    assert!(
        !row(&behind, "ndjson").contains('!'),
        "being behind is not trouble: {behind}"
    );

    assert!(
        silent.contains("not reported"),
        "an agent that says nothing about its buffers must not be drawn as an agent with \
         nothing in them: {silent}"
    );

    assert!(
        losing.contains("7 finding(s) were dropped"),
        "findings that were thrown away must not be read off a queue length: {losing}"
    );
    assert!(
        row(&losing, "ndjson").contains('!'),
        "a buffer that is dropping findings is trouble on the row, not a number in a column: \
         {losing}"
    );
    assert!(
        losing.contains("was never sent"),
        "and what that cost is said in words: {losing}"
    );
}

#[test]
fn the_history_that_does_not_fit_the_screen_is_named_as_kept_and_not_as_dropped() {
    let mut view = with_a_store();
    if let Some(status) = view.status.as_mut() {
        status.agent.findings.retained = 500;
        status.agent.findings.capacity = 500;
        status.agent.findings.total = 1_200;
        status.agent.findings.dropped = 700;
    }

    let page = drawn_from(&view, 200);

    assert!(page.contains("500 of 500 on the findings screen"), "{page}");
    assert!(page.contains("700 of them in the journal only"), "{page}");
    assert!(
        !page.contains("700 dropped"),
        "they are on disk and `jq` reads them: {page}"
    );
    assert!(
        !page.contains("raised"),
        "the word meant this run alone and the number no longer does: {page}"
    );
}

#[test]
fn a_store_with_nothing_in_it_is_named_as_empty_and_not_drawn_as_a_full_one() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.store = Some(vigil_model::StoreStatus {
            records: vigil_model::Counted {
                held: 0,
                ceiling: 10_000,
            },
            journal_path: Some("/var/lib/vigil/findings/journal.ndjson".into()),
            ..vigil_model::StoreStatus::default()
        });
    }

    let page = drawn_from(&view, 120);

    assert!(page.contains("Nothing recorded yet"), "{page}");
    assert!(page.contains("journal.ndjson"), "{page}");
}
