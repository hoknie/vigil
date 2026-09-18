use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::tests::harness::{app, drawn_at, number, press};
use crate::ui::fixture::screen;
use crate::ui::{Level, Reading};

fn on_the_containers() -> App {
    let mut app = app();
    press(&mut app, number(screen("containers")));
    drawn_at(&app, 120, 30);
    app
}

fn onto(app: &mut App, group: &str, list: &str) {
    for _ in 0..4 {
        if app.group_showing() == Some(group) {
            break;
        }
        press(app, KeyCode::Right);
    }
    assert_eq!(app.group_showing(), Some(group));
    press(app, KeyCode::Down);
    for _ in 0..8 {
        if app.pane().is_some_and(|pane| pane.name() == list) {
            break;
        }
        press(app, KeyCode::Right);
    }
    assert_eq!(
        app.pane().map(|pane| pane.name().to_string()),
        Some(list.to_string())
    );
    press(app, KeyCode::Down);
    assert_eq!(app.level, Level::List);
    drawn_at(app, 120, 30);
}

fn choose(app: &mut App, wanted: &str) {
    let Some(at) = app
        .chooser
        .offered()
        .iter()
        .position(|option| option == wanted)
    else {
        panic!("{wanted} is not offered: {:?}", app.chooser.offered());
    };
    while app.chooser.at() != at {
        press(app, KeyCode::Right);
    }
    press(app, KeyCode::Enter);
}

#[test]
fn the_containers_screen_offers_the_host_and_every_engine_on_its_first_row() {
    let app = on_the_containers();
    let page = drawn_at(&app, 80, 24);

    assert!(page.contains("[host]"), "{page}");
    assert!(
        page.contains(" docker ") && page.contains(" podman "),
        "{page}"
    );
    assert_eq!(
        app.group_showing(),
        Some("host"),
        "the screen opens on what /proc sees, as it did before the engines were read"
    );
}

#[test]
fn a_row_of_a_list_the_console_only_reads_offers_s_and_the_sheet_carries_the_key_of_its_findings() {
    let mut app = on_the_containers();
    onto(&mut app, "docker", "images");
    press(&mut app, KeyCode::Right);
    let page = drawn_at(&app, 120, 40);
    assert!(page.contains("[ S suppress it ]"), "{page}");

    press(&mut app, KeyCode::Char('S'));
    let sheet = drawn_at(&app, 120, 40);
    let row = app
        .pane_row_under_the_cursor()
        .expect("a row under the cursor")
        .key;

    assert!(
        sheet.contains(&format!("finding_key: \"engine|{row}\"")),
        "the entry is written under the key the rules raise findings about this row with, \
         which is the family and then the row: {sheet}"
    );
}

#[test]
fn the_docker_containers_narrow_to_one_compose_project_and_say_so_in_the_footer() {
    let mut app = on_the_containers();
    onto(&mut app, "docker", "containers");
    for _ in 0..4 {
        if app
            .pane_row_under_the_cursor()
            .is_some_and(|row| row.key.ends_with("shop-web-1"))
        {
            break;
        }
        press(&mut app, KeyCode::Down);
    }

    press(&mut app, KeyCode::Char('f'));
    choose(&mut app, "only project shop");

    assert_eq!(app.pane_keys().len(), 2, "{:?}", app.pane_keys());
    let page = drawn_at(&app, 120, 30);
    assert!(page.contains("only project shop"), "{page}");
}

#[test]
fn an_engine_that_is_not_installed_keeps_its_name_and_says_so_in_one_line() {
    let mut app = on_the_containers();
    app.view.readings.put(
        "containers-engines",
        Reading::Taken(vigil_engines::fixture::only_docker()),
    );
    for _ in 0..4 {
        if app.group_showing() == Some("podman") {
            break;
        }
        press(&mut app, KeyCode::Right);
    }

    let page = drawn_at(&app, 80, 24);

    assert!(page.contains("[podman]"), "{page}");
    assert!(
        page.contains("podman is not installed on this host."),
        "{page}"
    );
}

#[test]
fn the_lists_of_an_engine_are_drawn_whatever_list_of_the_host_the_cursor_was_on() {
    let app = on_the_containers();

    assert_eq!(
        app.shown_panes().len(),
        1,
        "the host group has one list, and the engines' lists are judged against their own \
         reading and not against the reading of the list the cursor happens to be on"
    );
    assert_eq!(app.every_shown_pane().len(), 15);
}

#[test]
fn a_list_the_engine_did_not_answer_for_says_so_where_the_table_would_be() {
    let mut app = on_the_containers();
    app.view.readings.put(
        "containers-engines",
        Reading::Taken(vigil_engines::fixture::docker_silent_on(
            vigil_engines::Subject::Image,
        )),
    );
    onto(&mut app, "docker", "images");

    let page = drawn_at(&app, 80, 24);

    assert!(
        page.contains("docker did not answer for its images."),
        "{page}"
    );
    assert!(
        !page.contains("TAGS"),
        "an engine that did not answer holds an unknown number of images, not none, and a \
         table with no rows under its headers says none: {page}"
    );
}

#[test]
fn every_list_of_a_merged_section_is_judged_against_its_own_reading_whatever_list_is_open() {
    let view = crate::ui::fixture::view();
    let section = crate::ui::holding("containers").expect("the containers screen");
    let search = crate::ui::Search::default();
    let mut buffer = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, 120, 24));

    crate::ui::screens::pane::render(
        &view,
        crate::ui::fixture::look(),
        section.as_ref(),
        &crate::ui::screens::pane::Showing {
            at: 0,
            search: &search,
            hidden: &[],
            cursor: 0,
            arrows: crate::ui::Arrows::Groups,
            group: Some("docker"),
            sorting: crate::ui::Sorting::default(),
            note: None,
            elsewhere: 0,
            gone: None,
            arranged: None,
            marked: Vec::new(),
            opened: Vec::new(),
            only: &[],
            listed: None,
            tally: None,
        },
        buffer.area,
        &mut buffer,
    );
    let page = crate::ui::helpers::words::text::to_text(&buffer);

    assert!(
        page.contains("images") && page.contains("registries"),
        "the list open is the host's, which reads /proc; docker's lists read the engines and \
         are drawn for docker whatever the open list reads: {page}"
    );
}
