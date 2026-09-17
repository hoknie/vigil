use ratatui::crossterm::event::KeyCode;

use crate::ui::Level;
use crate::ui::app::tests::harness::{app, drawn_at, number, press};
use crate::ui::fixture::screen;

const UNITS: usize = 0;

const CRON: usize = 2;

fn on_the_startup(pane: usize) -> crate::ui::App {
    let mut app = app();
    press(&mut app, number(screen("startup")));
    drawn_at(&app, 120, 30);
    for _ in 0..pane {
        press(&mut app, KeyCode::Right);
    }
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }
    drawn_at(&app, 120, 30);
    app
}

#[test]
fn the_band_over_a_unit_offers_the_six_words_and_says_what_enter_costs() {
    let mut app = on_the_startup(UNITS);

    press(&mut app, KeyCode::Char('U'));

    assert!(app.choosing(), "the cursor is on a unit, so there is one");
    let ways = drawn_at(&app, 120, 30);
    for word in [
        "stop it now",
        "start it now",
        "do not start it at the next boot",
        "start it at the next boot",
        "until it is unmasked",
        "take the mask off",
    ] {
        assert!(ways.contains(word), "{word} is not on the band: {ways}");
    }
    assert!(
        ways.contains("on this host, now"),
        "there is no second band after this one, so this is where Enter has to be spelled \
         out: {ways}"
    );
    assert!(
        ways.contains("C or Esc walks away"),
        "the way out of a band that stops a service is written on it: {ways}"
    );
}

#[test]
fn the_band_over_a_cron_job_offers_the_hash_and_nothing_of_systemd() {
    let mut app = on_the_startup(CRON);

    press(&mut app, KeyCode::Char('U'));

    assert!(app.choosing(), "{}", drawn_at(&app, 120, 30));
    let ways = drawn_at(&app, 120, 30);
    assert!(ways.contains("put a # in front of the line"), "{ways}");
    assert!(ways.contains("take the # off the line"), "{ways}");
    assert!(
        !ways.contains("boot"),
        "a crontab line is not a unit, and a word that answers every row with a refusal is \
         a word that must not be on the band: {ways}"
    );
}

#[test]
fn the_list_that_answers_to_it_says_so_at_the_foot_of_the_screen() {
    let units = drawn_at(&on_the_startup(UNITS), 120, 30);
    let cron = drawn_at(&on_the_startup(CRON), 120, 30);

    assert!(units.contains("U stop/disable"), "{units}");
    assert!(cron.contains("U comment out"), "{cron}");
    assert!(
        !units.contains("K close") && !units.contains("K stop"),
        "the key that signals a process belongs to the sockets and the running programs, \
         and offering it here would send a reader to a refusal: {units}"
    );
}

#[test]
fn on_a_list_that_starts_nothing_the_key_says_where_the_rows_it_acts_on_live() {
    let mut app = app();
    press(&mut app, number(screen("accounts")));
    drawn_at(&app, 120, 30);
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }

    press(&mut app, KeyCode::Char('U'));

    assert!(!app.choosing(), "{}", drawn_at(&app, 120, 30));
    for width in [80u16, 120] {
        let page = drawn_at(&app, width, 30);
        assert!(
            page.contains("startup screen"),
            "{width}: a key that answers with silence on the screen next to the one it works \
             on is a key a reader gives up on, and a sentence cut off before it names the \
             screen is the same silence: {page}"
        );
    }
}

#[test]
fn the_key_does_nothing_on_a_list_that_starts_and_stops_nothing() {
    let mut app = app();
    press(&mut app, number(screen("ports")));
    drawn_at(&app, 120, 30);
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }

    press(&mut app, KeyCode::Char('U'));

    assert!(
        !app.choosing(),
        "a socket is closed with K, and a band of systemd words over a list of sockets is a \
         band whose every letter ends in a refusal"
    );
}
