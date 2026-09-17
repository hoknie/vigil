use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::tests::harness::{a_configuration, drawn_at, number, press, watching};
use crate::ui::fixture::screen;
use crate::ui::helpers::finding::acts::Acts;
use crate::ui::{Level, Screen, holding};

fn keys_of(acts: Acts) -> Vec<char> {
    acts.buttons().iter().map(|button| button.key).collect()
}

fn on_the_watched_files() -> App {
    let path = a_configuration();
    let mut app = watching(&path);
    press(&mut app, number(screen("system")));
    drawn_at(&app, 120, 40);
    for _ in 0..5 {
        if app
            .pane()
            .is_some_and(|pane| pane.name() == "watched files")
        {
            while app.level != Level::List {
                press(&mut app, KeyCode::Down);
            }
            return app;
        }
        press(&mut app, KeyCode::Right);
    }
    panic!("no list of that section is the watched files");
}

#[test]
fn every_key_a_list_offers_on_a_row_is_a_button_in_the_detail_of_that_row() {
    for screen in Screen::all() {
        let Some(section) = holding(screen.name()) else {
            continue;
        };
        for pane in section.panes() {
            let offers = pane.offers();
            let buttons = keys_of(Acts::of_a_pane(&offers, true));

            for (offered, key, what) in [
                (offers.killing.is_some(), 'K', "close or stop the row"),
                (offers.controlling.is_some(), 'U', "start or stop it"),
                (offers.history, 'H', "the runs behind the row"),
                (offers.graph, 'P', "the path a packet takes"),
            ] {
                assert_eq!(
                    offered,
                    buttons.contains(&key),
                    "{} of {}: a key that works on a row is drawn among the buttons of that \
                     row's detail, and a button that is drawn answers with a deed rather than a \
                     refusal — {key} is meant to {what}",
                    pane.name(),
                    screen.name()
                );
            }
        }
    }
}

#[test]
fn a_row_nobody_can_act_on_is_offered_no_button_at_all() {
    for screen in Screen::all() {
        let Some(section) = holding(screen.name()) else {
            continue;
        };
        for pane in section.panes() {
            assert!(
                keys_of(Acts::of_a_pane(&pane.offers(), false)).is_empty(),
                "{} of {}: with the cursor on no row of the reading there is nothing to act on, \
                 and a button that acts on nothing is a button that answers with a refusal",
                pane.name(),
                screen.name()
            );
        }
    }
}

#[test]
fn the_watched_files_offer_their_three_keys_as_buttons_and_a_button_does_what_the_key_does() {
    let mut app = on_the_watched_files();
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    let page = drawn_at(&app, 120, 40);

    assert!(
        page.contains("ACTIONS")
            && page.contains("[ n new ]")
            && page.contains("[ e edit ]")
            && page.contains("[ D delete ]"),
        "the keys that write the configuration are on the list and in the detail of a row: {page}"
    );

    press(&mut app, KeyCode::Right);
    while app
        .acts()
        .button(app.button)
        .is_some_and(|button| button.key != 'e')
    {
        press(&mut app, KeyCode::Right);
    }
    press(&mut app, KeyCode::Enter);

    assert!(
        app.editing.is_some(),
        "the edit button opens the form the e key opens, because a button that carries a key's \
         name and does something else is worse than no button"
    );
}
