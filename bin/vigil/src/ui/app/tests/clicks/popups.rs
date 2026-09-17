use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::tests::harness::{app, click, drawn_at, into, press, where_it_says};
use crate::ui::fixture::screen;
use crate::ui::{Sorting, Target};

const WIDE: (u16, u16) = (120, 40);

fn sorting(app: &App) -> Sorting {
    app.sorted()
}

fn with_the_sort_band() -> App {
    let mut app = app();
    into(&mut app, screen("ports"), WIDE.0, WIDE.1);
    press(&mut app, KeyCode::Char('s'));
    drawn_at(&app, WIDE.0, WIDE.1);
    app
}

#[test]
fn a_click_on_an_option_of_a_band_chooses_it_and_closes_the_band() {
    let mut app = with_the_sort_band();
    let page = drawn_at(&app, WIDE.0, WIDE.1);
    let was = sorting(&app);
    let (column, row) = where_it_says(&page, "USER \u{2191}");

    click(&mut app, column, row);

    assert!(
        !app.choosing(),
        "the band answered the click and has nothing left to ask: {page}"
    );
    assert_ne!(
        sorting(&app),
        was,
        "the option under the pointer is the option chosen: {page}"
    );
    assert_eq!(
        sorting(&app),
        Sorting::of(5),
        "and it is the one drawn there, counted from the top of the band: {page}"
    );
}

#[test]
fn a_click_outside_an_open_band_puts_it_away_and_changes_nothing() {
    let mut app = with_the_sort_band();
    let page = drawn_at(&app, WIDE.0, WIDE.1);
    let was = sorting(&app);
    let cursor = app.panes().map(|panes| panes.at());

    click(&mut app, WIDE.0 - 2, WIDE.1 - 6);

    assert!(
        !app.choosing(),
        "a click beside a band is the reader walking away from it, which is Esc: {page}"
    );
    assert_eq!(
        sorting(&app),
        was,
        "walking away from a band leaves the list as it was"
    );
    assert_eq!(
        app.panes().map(|panes| panes.at()),
        cursor,
        "and the click that closed it must not also land on the row it was covering"
    );
}

#[test]
fn every_option_of_an_open_band_is_a_place_on_the_screen_that_can_be_clicked() {
    let app = with_the_sort_band();
    let page = drawn_at(&app, WIDE.0, WIDE.1);

    for said in [
        "as the agent sends it",
        "PROTO \u{2191}",
        "PROGRAM \u{2193}",
    ] {
        let (column, row) = where_it_says(&page, said);
        assert!(
            app.targets_under(column, row)
                .iter()
                .any(|target| matches!(target, Target::Aim(crate::ui::Aim::Option(_)))),
            "{said} is offered on the screen and nothing happens when it is clicked: {page}"
        );
    }
}
