use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::ui::app::App;
use crate::ui::fixture;
use crate::ui::helpers::words::text;
use crate::ui::{Audience, Screen};

use super::harness::{app, drawn, drawn_at, into, number, opened, press};
use crate::ui::fixture::screen;

#[test]
fn where_the_arrows_are_is_on_the_screen_and_readable_with_no_colour() {
    let mut app = app();
    press(&mut app, number(Screen::FINDINGS));

    let on_the_list = drawn_at(&app, 200, 24);
    assert!(
        on_the_list.contains("▸ What the agent has found"),
        "the caret is in the panel of the open section: {on_the_list}"
    );
    assert!(on_the_list.contains("│ > 09:00:00"), "{on_the_list}");

    press(&mut app, KeyCode::Enter);
    let beside_the_list = drawn_at(&app, 200, 24);
    assert!(
        beside_the_list.contains("▸ FINDINGS"),
        "the first press opened the panel and the arrows are still on the list, so the \
         caret has not moved: {beside_the_list}"
    );
    assert!(
        beside_the_list.contains("│ > 09:00:00"),
        "{beside_the_list}"
    );

    press(&mut app, KeyCode::Enter);
    let in_the_detail = drawn_at(&app, 200, 24);
    assert!(
        in_the_detail.contains("▸ THE SELECTED FINDING"),
        "{in_the_detail}"
    );
    assert!(in_the_detail.contains("  FINDINGS"), "{in_the_detail}");
    assert!(in_the_detail.contains("│ · 09:00:00"), "{in_the_detail}");
    assert!(!in_the_detail.contains(" > "), "{in_the_detail}");
}

#[test]
fn there_is_one_caret_on_the_page_and_the_row_of_screen_names_is_gone() {
    let mut app = app();
    press(&mut app, number(screen("accounts")));

    let page = drawn_at(&app, 200, 24);

    assert!(
        page.contains("▸ Who can log in"),
        "the caret is in the panel of the open section: {page}"
    );
    assert!(
        !page.contains("1 summary") && !page.contains("2 ports"),
        "the row of screen names along the top is gone: {page}"
    );
}

#[test]
fn a_script_asking_for_the_difference_gets_the_difference() {
    let options = opened(&[
        "capture",
        "--socket",
        "/nonexistent/vigil.sock",
        "--screen",
        "difference",
    ]);
    let mut app = App::new(
        &options,
        options.opening(Screen::SUMMARY),
        fixture::monochrome(),
        Audience::Script,
    );
    app.view = fixture::view();
    app.settle();

    let page = drawn_at(&app, 80, 400);

    assert!(page.contains("CHANGED"), "{page}");
    assert!(page.contains("suppressions:"), "{page}");
    assert!(
        !page.contains("shown of 3 held"),
        "and not the list: {page}"
    );
    assert!(
        page.contains("all ") && page.contains(" lines"),
        "and a file is told that nothing was left off the end of it: {page}"
    );
}

#[test]
fn a_printed_accounts_page_still_holds_every_object_the_collector_wrote() {
    let options = opened(&["capture", "--screen", "accounts"]);
    let mut app = App::new(
        &options,
        options.opening(Screen::SUMMARY),
        fixture::monochrome(),
        Audience::Script,
    );
    app.view = fixture::view();

    let mut buffer = Buffer::empty(Rect::new(0, 0, 120, 200));
    app.draw(buffer.area, &mut buffer);
    let page = text::to_text(&buffer);

    for object in [
        "contractor",
        "wheel",
        "%wheel",
        "SHA256:ie96zLdp",
        "backup",
        "pts/0",
    ] {
        assert!(page.contains(object), "{object} is not on the page: {page}");
    }
}

#[test]
fn a_printed_startup_page_holds_every_list_of_the_reading_and_not_only_the_first() {
    let options = opened(&["capture", "--screen", "startup"]);
    let mut app = App::new(
        &options,
        options.opening(Screen::SUMMARY),
        fixture::monochrome(),
        Audience::Script,
    );
    app.view = fixture::view();

    let mut buffer = Buffer::empty(Rect::new(0, 0, 120, 200));
    app.draw(buffer.area, &mut buffer);
    let page = text::to_text(&buffer);

    for object in [
        "nginx.service",
        "logrotate.timer",
        "implant",
        "overlay",
        "profile",
    ] {
        assert!(page.contains(object), "{object} is not on the page: {page}");
    }
}

#[test]
fn the_accounts_screen_draws_the_users_reading_and_not_the_ports_one() {
    let mut app = app();
    press(&mut app, number(screen("accounts")));

    let page = drawn(&app);

    assert_eq!(app.nav.at(), screen("accounts"));
    assert!(page.contains("ACCOUNT"), "{page}");
    assert!(page.contains("contractor"), "{page}");
    assert!(!page.contains("0.0.0.0:4444"), "{page}");
}

#[test]
fn every_screen_fits_every_terminal_this_is_read_on_at_every_level() {
    for screen in Screen::all().iter().chain([Screen::HOME].iter()) {
        for (width, height) in [(80u16, 24u16), (120, 40), (200, 60), (40, 10)] {
            for depth in 0..3 {
                let mut app = app();
                app.nav.visit(*screen);
                app.arrive();
                drawn_at(&app, width, height);
                for _ in 0..depth {
                    press(&mut app, KeyCode::Enter);
                }
                for line in drawn_at(&app, width, height).lines() {
                    assert!(
                        line.chars().count() <= width as usize,
                        "{} at {width}x{height}, {depth} in: {line}",
                        screen.name()
                    );
                }
            }
        }
    }
}

#[test]
fn the_programs_and_the_startup_sections_each_draw_their_own_reading() {
    let mut app = app();
    into(&mut app, screen("programs"), 120, 30);
    let programs = drawn_at(&app, 120, 30);
    assert!(programs.contains("nginx"), "{programs}");
    assert!(programs.contains("www-data"), "{programs}");

    into(&mut app, screen("startup"), 120, 30);
    let startup = drawn_at(&app, 120, 30);
    assert!(startup.contains("nginx.service"), "{startup}");
    assert!(!startup.contains("0.0.0.0:4444"), "{startup}");
}
