use ratatui::crossterm::event::KeyCode;

use super::harness::{app, drawn_at, into, number, press, typed};
use crate::ui::{Level, Screen, System};

#[test]
fn the_section_that_holds_two_readings_says_so_on_a_row_of_names() {
    let mut app = app();
    press(&mut app, number(Screen::System));

    assert_eq!(
        app.level,
        Level::Menu,
        "the row of names is a rung of its own"
    );
    let page = drawn_at(&app, 80, 30);
    assert!(page.contains("[the host]"), "{page}");
    assert!(page.contains("watched files"), "{page}");
}

#[test]
fn the_arrows_walk_from_one_reading_of_the_section_to_the_other() {
    let mut app = app();
    press(&mut app, number(Screen::System));

    press(&mut app, KeyCode::Right);

    assert_eq!(app.nav.lists.system.showing(), System::Files);
    let page = drawn_at(&app, 80, 30);
    assert!(page.contains("[watched files]"), "{page}");
    assert!(page.contains("/etc/ssh/sshd_config"), "{page}");

    press(&mut app, KeyCode::Left);
    assert_eq!(app.nav.lists.system.showing(), System::Host);
    assert!(drawn_at(&app, 80, 30).contains("memory and swap"));
}

#[test]
fn a_search_on_one_reading_of_the_section_does_not_narrow_the_other() {
    let mut app = app();
    into(&mut app, Screen::System, 80, 30);

    press(&mut app, KeyCode::Char('/'));
    typed(&mut app, "var");
    press(&mut app, KeyCode::Enter);
    let narrowed = drawn_at(&app, 80, 30);
    assert!(narrowed.contains("/var"), "{narrowed}");
    assert!(!narrowed.contains("memory and swap"), "{narrowed}");

    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Right);

    let other = drawn_at(&app, 80, 30);
    assert!(
        other.contains("/etc/hosts"),
        "the other list is whole: {other}"
    );
}

#[test]
fn each_reading_of_the_section_asks_the_agent_for_the_collector_that_holds_it() {
    let mut app = app();
    press(&mut app, number(Screen::System));

    assert_eq!(app.wanted_reading(), Some("resources"));

    press(&mut app, KeyCode::Right);

    assert_eq!(
        app.wanted_reading(),
        Some("files"),
        "a section of two readings asks for the one the reader is looking at, and not for both"
    );
}

#[test]
fn a_finding_about_a_filesystem_opens_the_host_and_one_about_a_file_opens_the_paths() {
    for (key, showing, row) in [
        ("resource|disk|/var", System::Host, "/var"),
        ("resource|boot", System::Host, "this host's boot"),
        ("file|/etc/hosts", System::Files, "/etc/hosts"),
    ] {
        let mut app = app();
        app.view.found.findings[0].finding_key = key.into();
        into(&mut app, Screen::Findings, 80, 30);

        press(&mut app, KeyCode::Char('o'));

        assert_eq!(app.nav.at(), Screen::System, "{key}");
        assert_eq!(app.nav.lists.system.showing(), showing, "{key}");
        let page = drawn_at(&app, 80, 30);
        assert!(
            page.lines()
                .any(|line| line.contains("> ") && line.contains(row)),
            "{key} landed somewhere else: {page}"
        );
    }
}

#[test]
fn a_finding_about_a_container_opens_the_row_about_the_container_it_names() {
    let mut app = app();
    app.view.found.findings[0].finding_key = "container|privileged|/usr/local/bin/agent".into();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), Screen::Containers);
    let page = drawn_at(&app, 120, 30);
    assert!(
        page.lines()
            .any(|line| line.contains("> ") && line.contains("9f2e8d7c6b5a")),
        "the key names the program the container runs, and that is the row it is about: {page}"
    );
}

#[test]
fn the_panel_says_every_value_the_agent_recorded_about_the_row_it_is_on() {
    let mut app = app();
    into(&mut app, Screen::System, 120, 40);
    press(&mut app, KeyCode::Right);

    let page = drawn_at(&app, 120, 40);
    assert!(page.contains("boot id"), "{page}");
    assert!(page.contains("1f0ec2b4"), "{page}");
    assert!(
        page.contains("booted at 2025-09-09"),
        "a moment the kernel counts in seconds is read as a day and a clock: {page}"
    );
    assert!(page.contains("object"), "{page}");
}

#[test]
fn what_a_script_is_given_carries_both_readings_of_the_section_without_a_key_being_pressed() {
    use crate::ui::app::App;
    use crate::ui::helpers::words::text;
    use crate::ui::{Audience, fixture};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    let options = super::harness::opened(&["capture", "--socket", "/nonexistent/vigil.sock"]);
    let mut app = App::new(
        &options,
        options.opening(Screen::System),
        fixture::monochrome(),
        Audience::Script,
    );
    app.view = fixture::view();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 400));
    app.draw(buffer.area, &mut buffer);

    let page = text::to_text(&buffer);
    assert!(page.contains("memory and swap"), "{page}");
    assert!(
        page.contains("[the host]"),
        "and which of the two readings it is: {page}"
    );
}

#[test]
fn the_key_that_puts_a_list_in_order_is_offered_on_the_new_sections_too() {
    for screen in [Screen::System, Screen::Containers] {
        let mut app = app();
        into(&mut app, screen, 80, 30);

        let page = drawn_at(&app, 80, 30);
        assert!(page.contains("s sort"), "{} : {page}", screen.name());

        press(&mut app, KeyCode::Char('s'));
        let choosing = drawn_at(&app, 80, 30);
        assert!(choosing.contains("sort by"), "{choosing}");
        assert!(choosing.contains("as the agent sends it"), "{choosing}");
    }
}

#[test]
fn every_section_of_this_console_is_opened_by_a_number_drawn_on_the_main_screen() {
    let mut app = app();

    for screen in Screen::ALL {
        let digit = screen
            .digit()
            .unwrap_or_else(|| panic!("{} has no number", screen.name()));
        press(
            &mut app,
            KeyCode::Char(char::from_digit(u32::from(digit), 10).expect("one of nine")),
        );

        assert_eq!(app.nav.at(), *screen, "{digit} opened something else");
    }
}

#[test]
fn no_text_this_console_draws_calls_the_node_it_watches_a_machine() {
    for screen in Screen::ALL.iter().chain([Screen::Home].iter()) {
        let mut app = app();
        press(&mut app, KeyCode::Char('?'));
        let help = drawn_at(&app, 120, 40);
        press(&mut app, KeyCode::Esc);

        into_or_onto(&mut app, *screen);
        let list = drawn_at(&app, 120, 40);
        press(&mut app, KeyCode::Char('d'));
        press(&mut app, KeyCode::Right);
        let panel = drawn_at(&app, 160, 40);

        for page in [&help, &list, &panel] {
            assert!(
                !page.to_lowercase().contains("machine"),
                "this product calls the node it watches a host, and the contract's own field \
                 is host_id: a screen that calls it a machine is the console speaking a \
                 second language. On {}: {page}",
                screen.name()
            );
        }
    }

    for screen in Screen::ALL.iter().chain([Screen::Home].iter()) {
        for said in [screen.name(), screen.title(), screen.holds()] {
            assert!(
                !said.to_lowercase().contains("machine"),
                "{}: {said}",
                screen.name()
            );
        }
    }
    for half in System::ALL {
        for said in [half.name(), half.caption(), half.detail()] {
            assert!(!said.to_lowercase().contains("machine"), "{said}");
        }
    }
}

fn into_or_onto(app: &mut crate::ui::app::App, screen: Screen) {
    match screen {
        Screen::Home => {
            drawn_at(app, 120, 40);
        }
        other => into(app, other, 120, 40),
    }
}
