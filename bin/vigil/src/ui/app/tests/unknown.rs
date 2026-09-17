use ratatui::crossterm::event::KeyCode;

use super::harness::{app, drawn_at, press};
use crate::ui::app::App;
use crate::ui::{Screen, fixture};

fn on_a_host_the_agent_reads_more_of() -> App {
    let mut app = app();
    let (status, reading) = fixture::of_a_reading_with_no_screen();
    app.view.status = Some(status);
    app.view.readings.put("kernel", reading);
    app.settle();
    app
}

fn onto_the_row(app: &mut App, named: &str) {
    for _ in 0..20 {
        if drawn_at(app, 120, 40)
            .lines()
            .any(|line| line.contains(" \u{25b8} ") && line.contains(named))
        {
            return;
        }
        press(app, KeyCode::Down);
    }
    panic!(
        "{named} is not a row of the main screen: {}",
        drawn_at(app, 120, 40)
    );
}

#[test]
fn a_reading_this_console_has_no_screen_for_opens_as_a_plain_list() {
    let mut app = on_a_host_the_agent_reads_more_of();
    onto_the_row(&mut app, "kernel");

    press(&mut app, KeyCode::Enter);

    assert_eq!(app.nav.at(), Screen::UNKNOWN);
    let page = drawn_at(&app, 120, 40);
    assert!(page.contains("vboxdrv"), "{page}");
    assert!(page.contains("overlay"), "{page}");
    assert!(
        page.contains("module"),
        "the class of each row is drawn, because this console knows nothing else about it: \
         {page}"
    );
}

#[test]
fn the_panel_of_such_a_row_holds_every_value_the_agent_recorded() {
    let mut app = on_a_host_the_agent_reads_more_of();
    onto_the_row(&mut app, "kernel");
    press(&mut app, KeyCode::Enter);

    press(&mut app, KeyCode::Right);

    let page = drawn_at(&app, 160, 40);
    assert!(page.contains("used by"), "{page}");
    assert!(page.contains("tainted"), "{page}");
    assert!(page.contains("module|"), "and which object it is: {page}");
}

#[test]
fn the_reader_may_search_it_like_any_other_list() {
    let mut app = on_a_host_the_agent_reads_more_of();
    onto_the_row(&mut app, "kernel");
    press(&mut app, KeyCode::Enter);

    press(&mut app, KeyCode::Char('/'));
    for letter in "vbox".chars() {
        press(&mut app, KeyCode::Char(letter));
    }
    press(&mut app, KeyCode::Enter);

    let page = drawn_at(&app, 120, 40);
    assert!(page.contains("vboxdrv"), "{page}");
    assert!(
        !page.contains("overlay"),
        "a list this console cannot name is still a list: {page}"
    );
}

#[test]
fn the_footer_says_which_reading_it_is_and_that_nothing_here_draws_it() {
    let mut app = on_a_host_the_agent_reads_more_of();
    onto_the_row(&mut app, "kernel");
    press(&mut app, KeyCode::Enter);

    let page = drawn_at(&app, 120, 40);

    assert!(
        page.contains("no screen of this build draws kernel"),
        "{page}"
    );
    assert!(
        page.contains("kernel: a reading no screen of this build draws"),
        "the line under the title names the reading, because nothing else on the page can: \
         {page}"
    );
}

#[test]
fn a_host_whose_agent_reads_only_what_this_console_draws_has_no_such_screen() {
    let app = app();

    assert!(
        !drawn_at(&app, 120, 40).contains("no screen draws it yet"),
        "the main screen carries no row for a reading that does not exist"
    );
}
