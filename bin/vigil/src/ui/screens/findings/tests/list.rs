use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_model::Severity;

use super::harness::{NOTHING_PICKED, drawn, floored, picking, searching_for, showing};
use crate::ui::helpers::words::text;
use crate::ui::screens::findings::{keys, render};
use crate::ui::{Audience, Dismissed, Filter, Look, fixture};

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
fn a_screen_emptied_by_what_this_console_silenced_does_not_read_as_a_quiet_host() {
    let view = fixture::view();
    let filter = Filter::default();
    let mut dismissed = Dismissed::default();
    dismissed.silence(["port.listen|tcp|0.0.0.0:4444".to_string()]);
    let mut buffer = Buffer::empty(Rect::new(0, 0, 100, 20));
    render(
        &view,
        fixture::look(),
        &picking(&filter, &NOTHING_PICKED, &dismissed),
        buffer.area,
        &mut buffer,
    );

    let page = text::to_text(&buffer);
    assert!(
        !page.contains("Nothing has been raised"),
        "three findings were raised and this console is hiding them: {page}"
    );
    assert!(page.contains("silenced from here"), "{page}");
    assert!(page.contains("3 row(s) hidden here"), "{page}");
    assert!(page.contains("Press u"), "{page}");
    assert!(page.contains("0 shown of 3 held"), "{page}");
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
        Look::new(fixture::monochrome(), Audience::Script),
        &showing(&Filter::default()),
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
