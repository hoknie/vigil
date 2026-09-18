use std::path::{Path, PathBuf};

use ratatui::crossterm::event::KeyCode;

use super::findings::{because, over_a_configuration};
use crate::ui::app::App;
use crate::ui::app::tests::harness::{click, drawn_at, into, press, watching};
use crate::ui::{Level, Screen};

fn apart() -> (App, String, PathBuf) {
    let (_, path) = over_a_configuration();
    std::fs::write(
        &path,
        "state_dir: /var/lib/vigil\nsuppressions_path: suppressions\nreporters: []\n",
    )
    .expect("writes");
    let directory = Path::new(&path)
        .parent()
        .expect("a directory")
        .join("suppressions");
    std::fs::create_dir_all(&directory).expect("a directory");
    std::fs::write(
        directory.join("10-deploy.yaml"),
        "suppressions:\n  - finding_key: \"user|group|docker\"\n    reason: the deploy user belongs there\n",
    )
    .expect("writes");

    let mut app = watching(&path);
    into(&mut app, Screen::FINDINGS, 200, 30);
    (app, path, directory)
}

fn to_the_silenced(app: &mut App) {
    while app.level != Level::Menu {
        press(app, KeyCode::Esc);
    }
    press(app, KeyCode::Right);
    press(app, KeyCode::Down);
}

#[test]
fn the_list_of_what_is_silenced_is_the_second_list_of_the_findings_like_the_lists_of_a_section() {
    let (mut app, _, _) = apart();

    press(&mut app, KeyCode::Esc);
    let menu = drawn_at(&app, 200, 30);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    let page = drawn_at(&app, 200, 30);

    assert_eq!(app.level, Level::List);
    assert!(
        menu.contains("[reported]") && menu.contains(" silenced "),
        "{menu}"
    );
    assert!(page.contains("[silenced]"), "{page}");
    assert!(page.contains("STANDING"), "{page}");
    assert!(page.contains("user|group|docker"), "{page}");
    assert!(page.contains("the deploy user belongs there"), "{page}");
    assert!(page.contains("10-deploy.yaml"), "{page}");
    assert!(page.contains("u report it again"), "{page}");

    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Down);
    let page = drawn_at(&app, 200, 30);
    assert!(!page.contains("STANDING"), "{page}");
    assert!(page.contains("A new listening port"), "{page}");
    assert_eq!(app.nav.at(), Screen::FINDINGS);
    assert_eq!(app.level, Level::List);
}

#[test]
fn the_silenced_list_neither_sorts_nor_narrows_and_says_why() {
    let (mut app, _, _) = apart();
    to_the_silenced(&mut app);

    for key in ['s', 'f', '/'] {
        press(&mut app, KeyCode::Char(key));
        let page = drawn_at(&app, 200, 30);
        assert!(page.contains("order of its files"), "{key}: {page}");
        assert!(!app.choosing(), "{key}");
    }
    press(&mut app, KeyCode::Char('x'));
    assert!(
        app.picked.is_empty(),
        "a row of a file is not a finding to pick"
    );
}

#[test]
fn what_the_agent_still_silences_from_its_start_is_on_the_list_even_when_no_file_holds_it() {
    let (mut app, _, _) = apart();

    to_the_silenced(&mut app);
    let page = drawn_at(&app, 200, 30);

    assert!(page.contains("port.listen|tcp|10.0.0.5:*"), "{page}");
    assert!(page.contains("still held"), "{page}");
    assert!(
        page.contains("not read yet"),
        "the entry in 10-deploy.yaml is not among what the agent said it silences: {page}"
    );
}

#[test]
fn silencing_a_finding_writes_into_the_consoles_own_file_and_the_list_shows_it_waiting() {
    let (mut app, path, directory) = apart();
    let configuration = std::fs::read_to_string(&path).expect("readable");

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "the staging api, expected here");
    let said = drawn_at(&app, 400, 30);
    to_the_silenced(&mut app);
    let page = drawn_at(&app, 200, 30);

    assert_eq!(
        std::fs::read_to_string(&path).expect("readable"),
        configuration,
        "the configuration `vigild configure --force` writes again holds nothing to lose"
    );
    assert!(directory.join("console.yaml").exists());
    assert!(said.contains("console.yaml"), "{said}");
    assert!(page.contains("port.listen|tcp|0.0.0.0:4444"), "{page}");
    assert!(page.contains("console.yaml"), "{page}");
}

#[test]
fn reporting_an_entry_again_takes_it_out_of_the_file_that_holds_it_and_only_that_entry() {
    let (mut app, _, directory) = apart();
    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "the staging api, expected here");
    to_the_silenced(&mut app);

    press(&mut app, KeyCode::Char('u'));
    let page = drawn_at(&app, 400, 30);

    let deploy = std::fs::read_to_string(directory.join("10-deploy.yaml")).expect("readable");
    assert_eq!(deploy, "suppressions: []\n");
    let console = std::fs::read_to_string(directory.join("console.yaml")).expect("readable");
    assert!(
        console.contains("port.listen|tcp|0.0.0.0:4444"),
        "{console}"
    );
    assert!(page.contains("taken out of"), "{page}");
    assert!(page.contains("10-deploy.yaml"), "{page}");
    assert!(page.contains("next round"), "{page}");
    assert!(!page.contains("the deploy user belongs there"), "{page}");
}

#[test]
fn an_entry_already_out_of_its_file_is_not_taken_out_twice_and_says_why() {
    let (mut app, _, directory) = apart();
    to_the_silenced(&mut app);
    press(&mut app, KeyCode::Char('G'));

    press(&mut app, KeyCode::Char('u'));
    let page = drawn_at(&app, 200, 30);

    assert!(page.contains("already out of its file"), "{page}");
    assert!(
        std::fs::read_to_string(directory.join("10-deploy.yaml"))
            .expect("readable")
            .contains("user|group|docker")
    );
}

#[test]
fn an_object_reported_again_comes_back_to_the_findings_this_console_took_it_off() {
    let (mut app, _, _) = apart();
    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");
    assert!(!drawn_at(&app, 200, 30).contains("A new listening port"));

    to_the_silenced(&mut app);
    press(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::Char('u'));
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Down);

    let page = drawn_at(&app, 200, 30);
    assert!(
        page.contains("A new listening port"),
        "the entry is out of the file, and hiding its rows still would be hiding what the \
         agent reports: {page}"
    );
}

#[test]
fn a_click_on_the_name_of_a_list_opens_it_as_on_every_row_of_lists() {
    let (mut app, _, _) = apart();
    let page = drawn_at(&app, 200, 30);
    let (row, line) = page
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains("silenced "))
        .expect("the row of lists is drawn");
    let column = line
        .chars()
        .take_while(|character| *character != 's')
        .count() as u16
        + 2;

    click(&mut app, column, row as u16);
    let page = drawn_at(&app, 200, 30);

    assert!(page.contains("[silenced]"), "{page}");
    assert!(page.contains("STANDING"), "{page}");
}
