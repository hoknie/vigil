use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::ui::helpers::layout::wrap;
use crate::ui::{Chooser, Look};

const MARGIN: usize = 4;

pub fn height(chooser: &Chooser, look: Look, width: u16) -> u16 {
    match chooser.choosing() {
        None => 0,
        Some(_) => lines(chooser, look, width).len() as u16,
    }
}

pub fn render(chooser: &Chooser, look: Look, area: Rect, buffer: &mut Buffer) {
    if chooser.choosing().is_none() || area.height == 0 {
        return;
    }
    Paragraph::new(lines(chooser, look, area.width)).render(area, buffer);
}

fn footing(what: crate::ui::Choosing, by_key: bool) -> &'static str {
    match (what.asks_before_acting(), by_key) {
        (true, true) => "a letter above does it, on this host, now · C or Esc walks away",
        (true, false) => {
            "← → choose · Enter does it, on this host, now · Esc leaves the host alone"
        }
        (false, _) => "← → choose · Enter apply · Esc leave it as it was",
    }
}

fn lines(chooser: &Chooser, look: Look, width: u16) -> Vec<Line<'static>> {
    let Some(what) = chooser.choosing() else {
        return Vec::new();
    };

    let mut spans = vec![Span::styled(
        format!(" {} ", what.caption()),
        look.palette.heading(),
    )];
    let mut lines = Vec::new();
    let mut used = what.caption().chars().count() + 2;
    let room = (width as usize).saturating_sub(MARGIN);

    for (at, option) in chooser.offered().iter().enumerate() {
        let named = match chooser.keys().get(at) {
            Some(key) => format!("{key} {option}"),
            None => option.clone(),
        };
        let drawn = match at == chooser.at() {
            true => format!("[{named}]"),
            false => format!(" {named} "),
        };
        if used + drawn.chars().count() + 1 > room && !spans.is_empty() {
            lines.push(Line::from(std::mem::take(&mut spans)));
            spans.push(Span::raw(" ".repeat(what.caption().chars().count() + 2)));
            used = what.caption().chars().count() + 2;
        }
        used += drawn.chars().count() + 1;
        spans.push(match at == chooser.at() {
            true => Span::styled(drawn, look.palette.selected()),
            false => Span::raw(drawn),
        });
        spans.push(Span::raw(" "));
    }
    if !spans.is_empty() {
        lines.push(Line::from(spans));
    }

    for part in wrap::wrap(
        footing(what, !chooser.keys().is_empty()),
        room.saturating_sub(1),
    ) {
        lines.push(Line::styled(format!("   {part}"), look.palette.quiet()));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::helpers::words::text;
    use crate::ui::{Choosing, fixture};

    fn drawn(at: usize, width: u16) -> String {
        let mut chooser = Chooser::default();
        chooser.open(
            Choosing::Sort,
            vec![
                "as the agent sends it".into(),
                "TIME ↑".into(),
                "TIME ↓".into(),
                "SEVERITY ↑".into(),
                "SEVERITY ↓".into(),
            ],
            at,
        );
        let tall = height(&chooser, fixture::look(), width);
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, tall.max(1)));
        render(&chooser, fixture::look(), buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn the_option_a_reader_is_on_is_marked_with_brackets_and_not_only_a_colour() {
        let page = drawn(1, 80);

        assert!(page.contains("[TIME ↑]"), "{page}");
        assert!(page.contains(" TIME ↓ "), "{page}");
    }

    #[test]
    fn it_says_which_keys_move_it_which_applies_and_which_puts_it_back() {
        let page = drawn(0, 80);

        assert!(page.contains("Enter apply"), "{page}");
        assert!(page.contains("Esc leave it as it was"), "{page}");
    }

    #[test]
    fn every_option_is_on_the_screen_at_eighty_columns_and_none_runs_off_the_side() {
        for width in [80u16, 100, 140] {
            let page = drawn(0, width);
            for option in ["as the agent sends it", "TIME ↑", "SEVERITY ↓"] {
                assert!(
                    page.contains(option),
                    "{width}: {option} is missing from {page}"
                );
            }
            for line in page.lines() {
                assert!(line.chars().count() <= width as usize, "{width}: {line}");
                assert!(!line.contains('…'), "{width}: {line}");
            }
        }
    }

    #[test]
    fn the_band_that_does_something_to_the_host_does_not_say_apply_like_the_others() {
        let mut chooser = Chooser::default();
        chooser.open_by_key(
            Choosing::Kill,
            vec![
                ('S', "ask the process to stop (SIGTERM)".into()),
                ('K', "stop the process now (SIGKILL)".into()),
            ],
        );
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 6));
        render(&chooser, fixture::look(), buffer.area, &mut buffer);
        let page = text::to_text(&buffer);

        assert!(page.contains("on this host, now"), "{page}");
        assert!(
            page.contains("C or Esc walks away"),
            "a reader looking for the way out of a destructive band must find it written \
             there, not remember it from the sort band: {page}"
        );
        assert!(
            page.contains("[S ask the process to stop (SIGTERM)]"),
            "it opens on the gentlest of them, and each option carries the letter that \
             picks it: {page}"
        );
        assert!(page.contains("K stop the process now"), "{page}");
    }

    #[test]
    fn a_console_choosing_nothing_draws_no_band_at_all() {
        let chooser = Chooser::default();

        assert_eq!(height(&chooser, fixture::look(), 80), 0);
    }
}
