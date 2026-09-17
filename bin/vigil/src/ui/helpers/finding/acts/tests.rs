use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget};

use super::*;
use crate::ui::fixture;
use crate::ui::helpers::words::text;

fn drawn(acts: Acts, at: Option<usize>, width: u16) -> String {
    let said = laid(acts, at, fixture::look(), width as usize).0;
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, said.len().max(1) as u16));
    Paragraph::new(said).render(buffer.area, &mut buffer);
    text::to_text(&buffer)
}

#[test]
fn the_panel_acts_on_the_one_record_it_is_showing_and_never_on_a_marked_set() {
    let page = drawn(Acts::of_a_row(Some(KillTarget::Socket)), None, 80);

    assert!(page.contains("[ K close this socket ]"), "{page}");
    assert!(page.contains("[ S suppress it ]"), "{page}");
    assert!(
        !page.contains("mark"),
        "marking belongs to the list, where a reader can see what is being gathered. \
         This panel is open on one row and acts on that row: {page}"
    );
}

#[test]
fn the_panel_of_a_program_offers_to_stop_it_and_not_to_close_a_socket_it_does_not_have() {
    let page = drawn(Acts::of_a_row(Some(KillTarget::Program)), None, 80);

    assert!(page.contains("[ K stop this program ]"), "{page}");
    assert!(page.contains("[ S suppress it ]"), "{page}");
    assert!(!page.contains("socket"), "{page}");
}

#[test]
fn an_account_row_offers_to_edit_and_delete_it_and_only_what_its_list_allows() {
    let both = drawn(Acts::of_an_account(true, true), None, 80);
    assert!(both.contains("[ e edit ]"), "{both}");
    assert!(both.contains("[ D delete ]"), "{both}");
    assert!(both.contains("[ S suppress it ]"), "{both}");
    assert!(
        !both.contains('K'),
        "an account is changed, not stopped: {both}"
    );

    let session = drawn(Acts::of_an_account(false, true), None, 80);
    assert!(!session.contains("edit"), "{session}");
    assert!(session.contains("[ D delete ]"), "{session}");

    assert_eq!(
        Acts::of_an_account(true, true)
            .button(1)
            .expect("a button")
            .key,
        'D'
    );
    assert!(
        Acts::of_an_account(false, false).buttons().is_empty(),
        "a row that can be neither edited nor deleted is not offered a suppression button \
         of its own here"
    );
}

#[test]
fn a_row_that_keeps_a_history_offers_it_as_a_button_and_after_what_acts_on_the_host() {
    let read_only = drawn(Acts::default().historied(true), None, 80);
    assert!(read_only.contains("[ H history ]"), "{read_only}");
    assert!(
        !read_only.contains("suppress"),
        "a row nothing can be done to is offered no suppression button here, and reading \
         its runs changes nothing: {read_only}"
    );

    let killable = Acts::of_a_row(Some(KillTarget::Program)).historied(true);
    assert_eq!(
        killable
            .buttons()
            .iter()
            .map(|button| button.key)
            .collect::<Vec<char>>(),
        vec!['K', 'S', 'H'],
        "what acts on the host comes first, because the arrows land on the first button"
    );
}

#[test]
fn a_row_nothing_can_be_done_to_draws_no_buttons_at_all() {
    assert!(
        laid(Acts::of_a_row(None), None, fixture::look(), 80)
            .0
            .is_empty(),
        "a button that answers with a refusal is worse than no button"
    );
}

#[test]
fn where_the_arrows_are_is_readable_on_a_terminal_with_no_colour_at_all() {
    let elsewhere = drawn(Acts::of_a_row(Some(KillTarget::Socket)), None, 80);
    let on_the_first = drawn(Acts::of_a_row(Some(KillTarget::Socket)), Some(0), 80);

    assert!(
        !elsewhere.contains('\u{25b8}'),
        "the arrows are somewhere else in the panel: {elsewhere}"
    );
    assert!(
        on_the_first.contains('\u{25b8}'),
        "and when they are here, the row says so: {on_the_first}"
    );
    assert!(
        on_the_first.contains("[ K close this socket ]")
            && !on_the_first.contains("[ S suppress it ]"),
        "one of them is chosen and the rest are not, and the brackets say which without \
         a colour: {on_the_first}"
    );
    assert!(
        elsewhere.contains("[ K close this socket ]") && elsewhere.contains("[ S suppress it ]"),
        "with the arrows elsewhere both are offers and neither is chosen: {elsewhere}"
    );
}

#[test]
fn the_arrows_walk_the_buttons_by_index_and_stop_at_the_ones_that_are_there() {
    assert_eq!(
        Acts::of_a_row(Some(KillTarget::Socket))
            .button(0)
            .expect("a button")
            .key,
        'K'
    );
    assert_eq!(
        Acts::of_a_row(Some(KillTarget::Socket))
            .button(1)
            .expect("a button")
            .key,
        'S'
    );
    assert!(Acts::of_a_row(Some(KillTarget::Socket)).button(2).is_none());
    assert!(Acts::of_a_row(None).button(0).is_none());
}

#[test]
fn a_button_row_wider_than_the_panel_wraps_instead_of_being_cut_off_mid_word() {
    for width in [24u16, 30, 40, 66, 80] {
        let page = drawn(Acts::of_a_row(Some(KillTarget::Socket)), None, width);

        assert!(
            page.contains("[ S suppress it ]"),
            "{width}: the last button was cut off the end: {page}"
        );
        for line in page.lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width}: {} columns: {line}",
                line.chars().count()
            );
        }
    }
}
