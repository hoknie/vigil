use ratatui::crossterm::event::KeyCode;

use super::findings::{because, over_a_configuration};
use crate::ui::app::tests::harness::{
    a_configuration, app, drawn_at, into, press, silenced, watching,
};
use crate::ui::{Deed, Screen};

#[test]
fn the_panel_of_a_finding_offers_the_deed_and_the_key_that_does_it() {
    let (mut app, path) = over_a_configuration();

    press(&mut app, KeyCode::Enter);

    let page = drawn_at(&app, 200, 30);
    assert!(page.contains("ACTIONS"), "{page}");
    assert!(page.contains(Deed::Remove.alone()), "{page}");

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    assert_eq!(
        silenced(&path).len(),
        1,
        "the key the panel offers does nothing from it"
    );
}

#[test]
fn the_file_a_finding_is_silenced_in_is_the_one_the_daemon_says_it_read() {
    let path = a_configuration();
    let mut app = app();
    if let Some(status) = app.view.status.as_mut() {
        status.agent.configuration_path = Some(path.clone());
    }
    into(&mut app, Screen::FINDINGS, 200, 30);

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    assert_eq!(
        silenced(&path).len(),
        1,
        "the daemon is started with the file named on its command line, and a console that \
         wrote the packaged default instead would silence nothing and say it had"
    );
}

#[test]
fn a_file_named_on_the_command_line_outranks_the_one_the_daemon_answered_with() {
    let named = a_configuration();
    let path = a_configuration();
    let mut app = watching(&named);
    if let Some(status) = app.view.status.as_mut() {
        status.agent.configuration_path = Some(path.clone());
    }
    into(&mut app, Screen::FINDINGS, 200, 30);

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    assert_eq!(silenced(&named).len(), 1);
    assert!(silenced(&path).is_empty());
}

#[test]
fn an_agent_that_does_not_say_which_file_it_read_is_answered_with_the_one_the_package_installs() {
    let mut app = app();
    if let Some(status) = app.view.status.as_mut() {
        status.agent.configuration_path = None;
    }
    into(&mut app, Screen::FINDINGS, 200, 30);

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    let page = drawn_at(&app, 200, 30);
    assert!(page.contains("/etc/vigil/vigil.yaml"), "{page}");
}

#[test]
fn the_summary_says_which_file_a_finding_would_be_silenced_in() {
    let mut app = app();
    press(&mut app, KeyCode::Char('8'));

    let page = drawn_at(&app, 200, 40);
    assert!(page.contains("configuration"), "{page}");
    assert!(page.contains("/etc/vigil/vigil.yaml"), "{page}");
    assert!(
        page.contains("silenced here"),
        "the file that key writes to is worth naming before the key is pressed: {page}"
    );
}
