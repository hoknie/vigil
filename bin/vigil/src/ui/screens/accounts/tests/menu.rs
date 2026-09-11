use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::harness::drawn;
use crate::ui::helpers::words::text as page;
use crate::ui::screens::accounts::{Showing, render};
use crate::ui::{Arrows, Search, Subject, fixture};

#[test]
fn the_submenu_is_on_the_screen_with_the_open_list_in_brackets() {
    let page = drawn(&fixture::view(), Subject::Groups);
    let row = page.lines().next().expect("a submenu");

    for name in ["users", "groups", "sudo", "keys", "ssh users", "logged in"] {
        assert!(row.contains(name), "{name} is not on the submenu: {row}");
    }
    assert!(row.contains("[groups]"), "{row}");
    assert!(!row.contains("[users]"), "{row}");
    assert!(row.chars().count() <= 80, "{row}");
}

#[test]
fn the_caret_says_when_the_arrows_are_on_the_submenu_and_not_in_the_list() {
    let view = fixture::view();
    let showing = |arrows| {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 20));
        render(
            &view,
            fixture::look(),
            &Showing {
                subject: Subject::Users,
                search: &Search::default(),
                cursor: 0,
                elsewhere: 0,
                arrows,
            },
            buffer.area,
            &mut buffer,
        );
        page::to_text(&buffer)
    };

    assert!(
        showing(Arrows::Menu).starts_with(" ▸ "),
        "{}",
        showing(Arrows::Menu)
    );
    assert!(!showing(Arrows::List).starts_with(" ▸ "));
    assert!(
        showing(Arrows::List)
            .lines()
            .any(|line| line.starts_with(" > ")),
        "the cursor is in the list instead: {}",
        showing(Arrows::List)
    );
}

#[test]
fn every_list_says_what_it_is_in_words() {
    let view = fixture::view();

    assert!(drawn(&view, Subject::SshUsers).contains("authorized_keys"));
    assert!(drawn(&view, Subject::SshUsers).contains("sshd"));
    assert!(drawn(&view, Subject::LoggedIn).contains("login records"));
    assert!(drawn(&view, Subject::Sudo).contains("/etc/sudoers"));
}

#[test]
fn each_list_has_columns_of_its_own_rather_than_four_that_fit_everything() {
    let view = fixture::view();

    assert!(drawn(&view, Subject::Users).contains("PASSWORD"));
    assert!(drawn(&view, Subject::Groups).contains("MEMBERS"));
    assert!(drawn(&view, Subject::Sudo).contains("MAY RUN"));
    assert!(drawn(&view, Subject::Keys).contains("FINGERPRINT"));
    assert!(drawn(&view, Subject::SshUsers).contains("KEYS"));
    assert!(drawn(&view, Subject::LoggedIn).contains("LINE"));
}
