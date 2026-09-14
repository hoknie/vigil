use ratatui::crossterm::event::KeyCode;
use vigil_model::AccountObject;

use crate::ui::app::App;
use crate::ui::app::tests::harness::{app, drawn_at, into, press, typed};
use crate::ui::fixture::screen;
use crate::ui::{Choosing, Level, Spot};

const DEPLOY: &str = "account|deploy";

fn on_the_users() -> App {
    let mut app = app();
    into(&mut app, screen("accounts"), 120, 30);
    app
}

fn cursor_to(app: &mut App, key: &str) {
    for _ in 0..40 {
        if app
            .pane_row_under_the_cursor()
            .is_some_and(|row| row.key == key)
        {
            return;
        }
        press(app, KeyCode::Down);
    }
    panic!("{key} is not on the list: {:?}", app.pane_keys());
}

#[test]
fn e_on_an_account_opens_a_form_in_place_of_the_list_and_not_over_it() {
    let mut app = on_the_users();
    cursor_to(&mut app, DEPLOY);

    press(&mut app, KeyCode::Char('e'));

    let editing = app.editing.as_ref().expect("a form is open");
    assert_eq!(editing.row(), Some(DEPLOY));
    let page = drawn_at(&app, 120, 30);
    assert!(page.contains("[ \u{2190} Back ]"), "{page}");
    assert!(
        page.contains("[ Save ]") && page.contains("[ Cancel ]"),
        "{page}"
    );
    assert!(page.contains("EDIT THE ACCOUNT deploy"), "{page}");
    assert!(
        !page.contains("ACCOUNTS") && !page.contains("THE SELECTED ACCOUNT"),
        "the form is a screen of its own, and the list is not drawn under it: {page}"
    );
    assert!(page.contains("Esc back"), "{page}");
}

#[test]
fn what_is_typed_and_left_with_esc_sends_nothing_and_returns_to_the_same_list() {
    let mut app = on_the_users();
    cursor_to(&mut app, DEPLOY);
    press(&mut app, KeyCode::Char('e'));
    typed(&mut app, "q/tmp");
    assert!(
        !app.leaving,
        "q typed into a field is a letter of the shell, not the key that leaves"
    );

    press(&mut app, KeyCode::Esc);

    assert!(app.editing.is_none());
    assert!(
        app.paper.is_none(),
        "nothing was asked, so nothing is reported"
    );
    assert_eq!(app.level, Level::List);
    assert_eq!(
        app.pane_row_under_the_cursor().map(|row| row.key),
        Some(DEPLOY.to_string())
    );
}

#[test]
fn the_back_button_at_the_top_leaves_the_form_as_esc_does() {
    let mut app = on_the_users();
    cursor_to(&mut app, DEPLOY);
    press(&mut app, KeyCode::Char('e'));
    while app.editing.as_ref().map(|editing| editing.spot()) != Some(Spot::Back) {
        press(&mut app, KeyCode::Up);
    }

    press(&mut app, KeyCode::Enter);

    assert!(app.editing.is_none());
}

#[test]
fn saving_a_form_nobody_changed_sends_nothing_and_says_so_on_the_form() {
    let mut app = on_the_users();
    cursor_to(&mut app, DEPLOY);
    press(&mut app, KeyCode::Char('e'));
    while app.editing.as_ref().map(|editing| editing.spot()) != Some(Spot::Save) {
        press(&mut app, KeyCode::Down);
    }

    press(&mut app, KeyCode::Enter);

    let editing = app.editing.as_ref().expect("the form stays open");
    assert!(
        editing
            .trouble()
            .is_some_and(|said| said.contains("othing")),
        "{:?}",
        editing.trouble()
    );
}

#[test]
fn a_form_that_could_not_reach_the_agent_stays_open_with_what_was_typed() {
    let mut app = on_the_users();
    cursor_to(&mut app, DEPLOY);
    press(&mut app, KeyCode::Char('e'));
    for _ in 0..9 {
        press(&mut app, KeyCode::Backspace);
    }
    typed(&mut app, "/bin/sh");
    while app.editing.as_ref().map(|editing| editing.spot()) != Some(Spot::Save) {
        press(&mut app, KeyCode::Down);
    }

    press(&mut app, KeyCode::Enter);

    let editing = app.editing.as_ref().expect("the form stays open");
    assert_eq!(editing.form().text("shell"), Some("/bin/sh"));
    assert!(
        editing.trouble().is_some_and(|said| !said.is_empty()),
        "and says why nothing happened"
    );
}

#[test]
fn d_on_an_account_opens_a_band_that_says_the_home_stays_and_c_walks_away() {
    let mut app = on_the_users();
    cursor_to(&mut app, DEPLOY);

    press(&mut app, KeyCode::Char('D'));

    assert_eq!(
        app.chooser.choosing(),
        Some(Choosing::Delete(AccountObject::User))
    );
    assert_eq!(app.chooser.keys(), &['D']);
    let page = drawn_at(&app, 120, 30);
    assert!(page.contains("home directory stays"), "{page}");
    assert!(page.contains("on this host, now"), "{page}");

    press(&mut app, KeyCode::Char('C'));
    assert_eq!(app.chooser.choosing(), None);
    assert!(
        app.paper.is_none(),
        "walking away asks the agent for nothing"
    );
}

#[test]
fn n_on_the_users_says_accounts_are_not_created_here() {
    let mut app = on_the_users();

    press(&mut app, KeyCode::Char('n'));

    assert!(app.editing.is_none());
    assert!(
        app.message
            .as_deref()
            .is_some_and(|said| said.contains("not created")),
        "{:?}",
        app.message
    );
}

#[test]
fn the_list_of_users_offers_the_keys_that_change_it_at_the_bottom() {
    let mut app = on_the_users();
    cursor_to(&mut app, DEPLOY);

    let page = drawn_at(&app, 200, 30);

    assert!(page.contains("e edit"), "{page}");
    assert!(page.contains("D delete"), "{page}");
    assert!(!page.contains("K close"), "{page}");
}

#[test]
fn the_panel_of_an_account_carries_edit_and_delete_buttons_and_enter_on_edit_opens_the_form() {
    let mut app = on_the_users();
    cursor_to(&mut app, DEPLOY);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    drawn_at(&app, 160, 40);
    while app.level != Level::Detail {
        press(&mut app, KeyCode::Right);
    }

    let page = drawn_at(&app, 160, 40);
    assert!(page.contains("[ e edit ]"), "{page}");
    assert!(page.contains("[ D delete ]"), "{page}");

    press(&mut app, KeyCode::Right);
    assert_eq!(app.button_at(), Some(0));
    press(&mut app, KeyCode::Enter);

    assert!(app.editing.is_some(), "{:?}", app.message);
}

#[test]
fn a_key_that_changes_accounts_says_where_it_works_on_a_list_that_changes_nothing() {
    let mut app = app();
    into(&mut app, screen("ports"), 120, 30);

    press(&mut app, KeyCode::Char('e'));

    assert!(app.editing.is_none());
    assert!(
        app.message
            .as_deref()
            .is_some_and(|said| said.contains("accounts")),
        "{:?}",
        app.message
    );
}
