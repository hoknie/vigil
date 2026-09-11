use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_model::Severity;

use crate::ui::helpers::words::text;
use crate::ui::screens::findings::{keys, render};
use crate::ui::{Audience, Filter, Look, View, fixture};

fn drawn(view: &View, filter: &Filter, width: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 20));
    render(
        view,
        filter,
        fixture::look(),
        0,
        true,
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}

fn floored(levels: usize) -> Filter {
    let mut filter = Filter::default();
    for _ in 0..levels {
        filter.move_floor(1);
    }
    filter
}

fn searching_for(wanted: &str) -> Filter {
    let mut filter = Filter::default();
    filter.search_mut().start();
    for character in wanted.chars() {
        filter.search_mut().type_character(character);
    }
    filter.search_mut().accept();
    filter
}

#[test]
fn a_finding_says_when_how_loud_what_kind_and_what_happened() {
    let page = drawn(&fixture::view(), &Filter::default(), 80);

    assert!(page.contains("09:00:00"), "{page}");
    assert!(
        page.contains("critical"),
        "the level is a word, not a colour: {page}"
    );
    assert!(page.contains("port.listen.new"), "{page}");
}

#[test]
fn a_wide_terminal_shows_the_object_behind_the_sentence() {
    let wide = drawn(&fixture::view(), &Filter::default(), 200);
    let narrow = drawn(&fixture::view(), &Filter::default(), 80);

    assert!(wide.contains("port.listen|tcp|0.0.0.0:4444"), "{wide}");
    assert!(wide.contains("OBJECT"), "{wide}");
    assert!(!narrow.contains("OBJECT"), "{narrow}");
}

#[test]
fn beside_the_panel_the_kind_gives_way_before_the_title_does() {
    let page = drawn(&fixture::view(), &Filter::default(), 60);

    assert!(page.contains("A new listening port on"), "{page}");
    assert!(
        page.contains("critical"),
        "the level survives everything: {page}"
    );
    assert!(!page.contains("KIND"), "{page}");
    for line in page.lines() {
        assert!(line.chars().count() <= 60, "{line}");
    }
}

#[test]
fn the_severity_floor_hides_the_quiet_ones_and_says_that_it_is_doing_so() {
    let page = drawn(&fixture::view(), &floored(4), 80);

    assert!(page.contains("critical and above"), "{page}");
    assert!(!page.contains("A user logged in"), "{page}");
}

#[test]
fn a_search_narrows_the_list_and_stays_on_the_screen_while_it_does() {
    let page = drawn(&fixture::view(), &searching_for("logged in"), 80);

    assert!(page.contains("A user logged in"), "{page}");
    assert!(!page.contains("A new listening port"), "{page}");
    assert!(
        page.contains("search \"logged in\""),
        "a filter that is not on the screen is a filter that lies about the host: {page}"
    );
    assert!(
        page.contains("every value the agent read"),
        "and it says what it looked at: {page}"
    );
}

#[test]
fn while_the_search_box_is_open_it_says_so_and_shows_what_is_in_it() {
    let mut filter = Filter::default();
    filter.search_mut().start();
    filter.search_mut().type_character('n');
    filter.search_mut().type_character('c');

    let page = drawn(&fixture::view(), &filter, 80);

    assert!(page.contains("search nc"), "{page}");
}

#[test]
fn a_list_emptied_by_the_readers_own_filter_does_not_read_as_a_quiet_host() {
    let page = drawn(&fixture::view(), &searching_for("nothing like this"), 80);

    assert!(page.contains("not by the agent"), "{page}");
    assert!(!page.contains("Nothing has been reported"), "{page}");
}

#[test]
fn a_refused_question_is_not_drawn_as_a_quiet_host_either() {
    let mut view = fixture::view();
    view.found.findings.clear();
    view.found.refused = Some("this build reads status, snapshot, findings".into());

    let page = drawn(&view, &Filter::default(), 80);

    assert!(page.contains("findings were refused"), "{page}");
    assert!(!page.contains("Nothing has been reported"), "{page}");
}

fn storing(store: vigil_model::StoreStatus) -> View {
    let mut view = fixture::view();
    view.found.findings.clear();
    if let Some(status) = view.status.as_mut() {
        status.agent.findings.total = 0;
        status.agent.store = Some(store);
    }
    view
}

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

#[test]
fn a_severity_this_build_does_not_know_is_shown_as_the_word_the_agent_used() {
    let mut view = fixture::view();
    let mut strange = fixture::finding("something from a newer agent", Severity::Info);
    strange.severity = Severity::Unknown("catastrophic".into());
    view.found.findings.push(strange);

    assert!(drawn(&view, &floored(4), 80).contains("catastrophic"));
}

#[test]
fn the_keys_are_the_event_ids_so_the_cursor_holds_one_finding_and_not_one_row_number() {
    let view = fixture::view();
    let passing = Filter::default().passing(&view.found.findings);

    assert_eq!(keys(&passing).len(), 3);
    assert!(keys(&passing).iter().all(|key| key.starts_with("0199a1b2")));
}

#[test]
fn a_files_page_puts_the_tally_under_the_table_rather_than_at_the_foot_of_a_tall_page() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 400));
    render(
        &fixture::view(),
        &Filter::default(),
        Look::new(fixture::monochrome(), Audience::Script),
        0,
        true,
        buffer.area,
        &mut buffer,
    );

    let page = text::to_text(&buffer);
    let lines: Vec<&str> = page.lines().collect();
    assert!(lines.len() < 10, "{} lines of page: {page}", lines.len());
    assert!(lines.last().expect("a page").contains("shown of"), "{page}");
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for width in [80u16, 120, 200] {
        for line in drawn(&fixture::view(), &Filter::default(), width).lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width} columns: {line}"
            );
        }
    }
}
