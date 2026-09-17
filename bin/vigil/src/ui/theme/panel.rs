use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType};

use super::caption::Keys;
use crate::ui::Look;

pub fn block(look: Look, name: &str, keys: Keys, arrows: bool) -> Block<'static> {
    let marker = match keys {
        Keys::Here => "\u{25b8} ",
        _ => "",
    };
    Block::bordered()
        .border_type(match arrows {
            true => BorderType::Thick,
            false => BorderType::Rounded,
        })
        .border_style(match arrows {
            true => look.palette.focus(),
            false => look.palette.border(),
        })
        .title_top(Line::styled(
            format!(" {marker}{name} "),
            look.palette.heading(),
        ))
}

pub fn tail(block: Block<'static>, look: Look, tail: &str) -> Block<'static> {
    match tail.is_empty() {
        true => block,
        false => {
            block.title_top(Line::styled(format!(" {tail} "), look.palette.quiet()).right_aligned())
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn drawn(name: &str, keys: Keys, arrows: bool, width: u16) -> String {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, 3));
        tail(
            block(fixture::look(), name, keys, arrows),
            fixture::look(),
            "all 12 lines",
        )
        .render(buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn a_panel_is_drawn_with_rounded_corners_and_its_name_in_the_top_edge() {
        let page = drawn("FINDINGS", Keys::Elsewhere, false, 40);
        let top = page.lines().next().unwrap_or_default();

        assert!(
            top.starts_with("\u{256d} FINDINGS \u{2500}"),
            "the name sits in the edge of a rounded frame, not on a line of its own: {page}"
        );
        assert!(
            top.ends_with(" all 12 lines \u{256e}"),
            "and how far down it the reader is sits on the right of the same edge: {page}"
        );
        assert!(
            page.lines()
                .last()
                .unwrap_or_default()
                .starts_with('\u{2570}'),
            "{page}"
        );
    }

    #[test]
    fn the_panel_the_arrows_move_in_is_told_by_the_thickness_of_its_lines_with_no_colour() {
        let here = drawn("FINDINGS", Keys::Here, true, 40);
        let elsewhere = drawn("FINDINGS", Keys::Elsewhere, false, 40);

        assert!(
            here.starts_with("\u{250f} \u{25b8} FINDINGS \u{2501}"),
            "a monochrome terminal has only the characters to say where the arrows are: {here}"
        );
        assert!(
            !elsewhere.contains('\u{2501}') && !elsewhere.contains('\u{2503}'),
            "and a panel without them must not borrow the thick line: {elsewhere}"
        );
    }

    #[test]
    fn with_no_colour_the_frame_with_the_arrows_is_not_painted_to_tell_it_apart() {
        for arrows in [true, false] {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 40, 3));
            block(fixture::look(), "FINDINGS", Keys::Here, arrows).render(buffer.area, &mut buffer);

            for cell in buffer.content() {
                assert_eq!(
                    (cell.fg, cell.bg),
                    (ratatui::style::Color::Reset, ratatui::style::Color::Reset),
                    "{arrows}: {:?} is coloured on a terminal that asked for none, so the \
                     thickness is no longer the only thing telling the two panels apart",
                    cell.symbol()
                );
            }
        }
    }

    #[test]
    fn a_panel_narrower_than_its_name_never_runs_past_its_own_corners() {
        for width in [8u16, 16, 24, 80] {
            let page = drawn("THE SELECTED FINDING", Keys::Here, true, width);
            for line in page.lines() {
                assert!(
                    line.chars().count() <= width as usize,
                    "{width}: {line} ({} columns)",
                    line.chars().count()
                );
            }
        }
    }
}
