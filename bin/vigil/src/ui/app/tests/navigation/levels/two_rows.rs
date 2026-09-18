use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::tests::harness::{app, drawn_at, number, press};
use crate::ui::fixture::grouped::{DOCKER, HOST, NOT_INSTALLED, PODMAN, SECTION, section};
use crate::ui::fixture::screen;
use crate::ui::sections::instead;
use crate::ui::{Level, Screen};

fn on_the_engines() -> App {
    let mut app = app();
    press(&mut app, number(screen(SECTION)));
    drawn_at(&app, 120, 30);
    app
}

fn onto_the_group(app: &mut App, named: &str) {
    for _ in 0..4 {
        if app.group_showing() == Some(named) {
            return;
        }
        press(app, KeyCode::Right);
    }
    panic!("{named} is not a group of this section");
}

#[test]
fn a_section_with_groups_is_entered_on_the_groups_and_the_arrows_walk_that_row() {
    let _standing = instead::drawn(SECTION, section);
    let mut app = on_the_engines();

    assert_eq!(app.level, Level::Groups);
    assert_eq!(app.group_showing(), Some(HOST));

    press(&mut app, KeyCode::Right);
    assert_eq!(app.group_showing(), Some(DOCKER));
    assert_eq!(app.nav.at(), screen(SECTION), "the section did not change");

    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Left);
    assert_eq!(app.group_showing(), Some(PODMAN), "and the row wraps");
}

#[test]
fn the_down_arrow_goes_from_the_groups_to_the_lists_and_then_into_the_list() {
    let _standing = instead::drawn(SECTION, section);
    let mut app = on_the_engines();
    onto_the_group(&mut app, DOCKER);

    press(&mut app, KeyCode::Down);
    assert_eq!(app.level, Level::Menu, "the second row is the lists");
    press(&mut app, KeyCode::Right);
    assert_eq!(
        app.pane().expect("a list of the group").name(),
        "volumes",
        "the arrows walk the lists of the chosen group and not the lists of the section"
    );

    press(&mut app, KeyCode::Down);
    assert_eq!(app.level, Level::List);

    press(&mut app, KeyCode::Up);
    assert_eq!(app.level, Level::Menu, "and the way back is the way in");
    press(&mut app, KeyCode::Up);
    assert_eq!(app.level, Level::Groups);
    press(&mut app, KeyCode::Up);
    assert_eq!(
        app.nav.at(),
        Screen::HOME,
        "above the groups is the way out of the section"
    );
}

#[test]
fn the_group_a_reader_chose_is_remembered_the_way_the_chosen_list_is() {
    let _standing = instead::drawn(SECTION, section);
    let mut app = on_the_engines();
    onto_the_group(&mut app, DOCKER);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Right);
    let list = app.pane().expect("a list of the group").name().to_string();

    press(&mut app, number(Screen::FINDINGS));
    press(&mut app, number(screen(SECTION)));

    assert_eq!(app.group_showing(), Some(DOCKER));
    assert_eq!(
        app.pane().expect("a list of the group").name(),
        list,
        "the section remembers which list of which group was open"
    );
}

#[test]
fn a_group_with_nothing_to_show_keeps_its_name_on_the_first_row_and_says_so_on_the_second() {
    let _standing = instead::drawn(SECTION, section);
    let mut app = on_the_engines();
    onto_the_group(&mut app, PODMAN);

    let page = drawn_at(&app, 120, 30);

    assert!(
        page.contains("[podman]"),
        "a group whose lists have nothing to show is still a group of this host: {page}"
    );
    assert!(
        page.contains(NOT_INSTALLED),
        "and the second row says why, where an empty table would say nothing: {page}"
    );
    assert!(
        !page.contains("NAME"),
        "no table is drawn under a sentence that explains why there is none: {page}"
    );
}

#[test]
fn the_lists_of_the_chosen_group_are_the_only_ones_on_the_second_row() {
    let _standing = instead::drawn(SECTION, section);
    let mut app = on_the_engines();

    let host = drawn_at(&app, 120, 30);
    assert!(host.contains("[containers]"), "{host}");
    assert!(
        !host.contains("images"),
        "the lists of another engine are not drawn beside this one: {host}"
    );

    onto_the_group(&mut app, DOCKER);
    let docker = drawn_at(&app, 120, 30);
    assert!(docker.contains("[images]"), "{docker}");
    assert!(docker.contains("volumes"), "{docker}");
    assert!(docker.contains("compose"), "{docker}");
}

#[test]
fn both_rows_and_the_keys_under_them_fit_a_terminal_of_eighty_columns() {
    let _standing = instead::drawn(SECTION, section);
    let mut app = on_the_engines();
    onto_the_group(&mut app, DOCKER);

    for width in [80u16, 120] {
        let page = drawn_at(&app, width, 24);
        for line in page.lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width} columns: {line}"
            );
        }
        assert!(page.contains("[docker]"), "{width} columns: {page}");
        assert!(page.contains("[images]"), "{width} columns: {page}");
    }
}

#[test]
fn the_hint_line_names_the_keys_of_the_row_the_reader_is_standing_on() {
    let _standing = instead::drawn(SECTION, section);
    let mut app = on_the_engines();

    let groups = drawn_at(&app, 120, 30);
    assert!(groups.contains("which group"), "{groups}");

    onto_the_group(&mut app, DOCKER);
    press(&mut app, KeyCode::Down);
    let lists = drawn_at(&app, 120, 30);
    assert!(lists.contains("which list"), "{lists}");
}

#[test]
fn a_section_whose_lists_belong_to_no_group_is_drawn_with_one_row_as_it_always_was() {
    let mut app = app();
    press(&mut app, number(screen("accounts")));

    assert_eq!(
        app.level,
        Level::Menu,
        "a section of this build is entered on its row of lists, not on a row of groups"
    );
    assert!(app.groups().is_empty());

    let page = drawn_at(&app, 80, 24);
    let lines: Vec<&str> = page.lines().collect();
    let row = lines
        .iter()
        .position(|line| line.contains("[users]"))
        .expect("the row of lists");

    assert!(
        lines[row - 1].starts_with('\u{250f}'),
        "the row of names sits directly under the edge of the section, with no row of groups \
         drawn over it: {page}"
    );
}

fn under_the_menu(page: &str) -> Vec<String> {
    page.lines()
        .skip_while(|line| !line.contains("what the engine holds"))
        .take_while(|line| !line.starts_with('\u{2517}'))
        .map(|line| line.trim_matches(['\u{2503}', '\u{2502}', ' ']).to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

#[test]
fn the_same_lists_without_their_groups_draw_the_page_this_console_draws_today() {
    let grouped = {
        let _standing = instead::drawn(SECTION, section);
        let app = on_the_engines();
        drawn_at(&app, 120, 30)
    };
    let flat = {
        let _standing = instead::drawn(SECTION, crate::ui::fixture::grouped::flat);
        let app = on_the_engines();
        drawn_at(&app, 120, 30)
    };

    assert!(
        !flat.contains("[host]"),
        "lists that belong to no group are drawn under no row of groups: {flat}"
    );
    assert!(
        flat.contains("[containers]") && flat.contains("images"),
        "they are all on the one row, as they are on every screen of this build: {flat}"
    );
    assert_eq!(
        under_the_menu(&flat),
        under_the_menu(&grouped),
        "the second level is a row above the screen and nothing below it: the same list of \
         the same reading is drawn line for line either way\n{flat}\n{grouped}"
    );
}

#[test]
fn the_keys_of_the_row_of_groups_are_named_on_a_terminal_of_eighty_columns_too() {
    let _standing = instead::drawn(SECTION, section);
    let app = on_the_engines();

    for width in [80u16, 120] {
        let page = drawn_at(&app, width, 24);
        let hint = page.lines().last().unwrap_or_default();

        assert!(
            hint.contains("which group"),
            "the reader is standing on the row of groups and is told about the list under it \
             instead, because the line naming the keys of this row was too long to fit \
             {width} columns: {hint}"
        );
    }
}

#[test]
fn what_a_script_is_given_holds_the_lists_of_every_group_and_not_only_the_chosen_one() {
    use crate::ui::app::tests::harness::opened;
    use crate::ui::{Audience, fixture};

    let _standing = instead::drawn(SECTION, section);
    let asked = opened(&["capture", "--socket", "/nonexistent/vigil.sock"]);
    let mut app = App::new(
        &asked,
        asked.opening(screen(SECTION)),
        fixture::monochrome(),
        Audience::Script,
    );
    app.view = fixture::view();
    let mut buffer = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 400));
    app.draw(buffer.area, &mut buffer);
    let page = crate::ui::to_text(&buffer);

    for named in ["containers", "images", "volumes", "networks", "compose"] {
        assert!(
            page.contains(&format!("[{named}]")),
            "a script cannot press an arrow, so every list of every group is printed for it, \
             and {named} is not: {page}"
        );
    }
}
