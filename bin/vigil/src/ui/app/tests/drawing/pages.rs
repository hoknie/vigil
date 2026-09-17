use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::ui::app::App;
use crate::ui::fixture;
use crate::ui::helpers::words::text;
use crate::ui::{Audience, Screen};

use crate::ui::app::tests::harness::{app, drawn, drawn_at, into, number, opened, press};
use crate::ui::fixture::screen;

#[test]
fn where_the_arrows_are_is_on_the_screen_and_readable_with_no_colour() {
    let mut app = app();
    press(&mut app, number(Screen::FINDINGS));

    let on_the_list = drawn_at(&app, 200, 24);
    assert!(
        on_the_list.contains("\u{250f} \u{25b8} What the agent has found"),
        "the caret and the thick frame are on the panel of the open section: {on_the_list}"
    );
    assert!(
        on_the_list.contains("\u{2503} \u{25b8} 09:00:00"),
        "{on_the_list}"
    );

    press(&mut app, KeyCode::Enter);
    let beside_the_list = drawn_at(&app, 200, 24);
    assert!(
        beside_the_list.contains("\u{250f} \u{25b8} FINDINGS"),
        "the first press opened the panel and the arrows are still on the list, so the \
         caret and the thick frame have not moved: {beside_the_list}"
    );
    assert!(
        beside_the_list
            .lines()
            .nth(1)
            .is_some_and(|line| line.starts_with(" What the agent has found")),
        "over two framed panes the section is named on a plain line and gives the thick line \
         up to the pane that has the arrows: {beside_the_list}"
    );
    assert!(
        beside_the_list.contains("\u{2503} \u{25b8} 09:00:00"),
        "{beside_the_list}"
    );

    press(&mut app, KeyCode::Enter);
    let in_the_detail = drawn_at(&app, 200, 24);
    assert!(
        in_the_detail.contains("\u{250f} \u{25b8} THE SELECTED FINDING"),
        "{in_the_detail}"
    );
    assert!(
        in_the_detail.contains("\u{256d} FINDINGS"),
        "the list keeps its name and gives up the caret and the thick line: {in_the_detail}"
    );
    assert!(
        in_the_detail.contains("\u{2502} \u{b7} 09:00:00"),
        "{in_the_detail}"
    );
    assert_eq!(
        in_the_detail.matches('\u{250f}').count(),
        1,
        "one thick frame on the page, or the reader cannot tell where the arrows are: \
         {in_the_detail}"
    );
}

#[test]
fn a_list_writes_its_tally_into_the_bottom_edge_of_the_frame_it_sits_in() {
    let mut app = app();
    press(&mut app, number(Screen::FINDINGS));

    for (width, height) in [(80u16, 24u16), (120, 30)] {
        let page = drawn_at(&app, width, height);
        let edge = page
            .lines()
            .find(|line| line.starts_with('\u{2517}'))
            .unwrap_or_default();

        assert!(
            edge.starts_with("\u{2517} 3 shown of 3 held") && edge.ends_with('\u{251b}'),
            "{width}x{height}: the tally is the footer of the frame and the frame is still \
             closed after it: {page}"
        );
        assert_eq!(
            page.matches("3 shown of 3 held").count(),
            1,
            "and it is not said a second time inside the frame: {page}"
        );
    }
}

#[test]
fn a_pane_alone_on_a_narrow_terminal_carries_the_thick_frame_under_a_plain_heading() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 24);
    press(&mut app, KeyCode::Enter);

    let page = drawn_at(&app, 80, 24);

    assert!(
        page.lines()
            .nth(1)
            .is_some_and(|line| line.starts_with(" What the agent has found")),
        "the section is named on a line of its own, not in a frame around the pane: {page}"
    );
    assert!(
        page.lines()
            .nth(2)
            .is_some_and(|line| line.starts_with("\u{250f} \u{25b8} THE SELECTED FINDING")),
        "the one pane on the screen has the arrows, so it has the thick frame and the caret, \
         and the frame starts at the edge of the terminal: {page}"
    );
    for line in page.lines() {
        assert!(line.chars().count() <= 80, "{line}");
    }
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
