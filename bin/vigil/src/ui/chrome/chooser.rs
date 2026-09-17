use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use crate::ui::chrome::popup;
use crate::ui::helpers::layout::popup as placing;
use crate::ui::{Aim, Chooser, Look};

const BORDERS: u16 = 2;

const LEAD: u16 = 1;

pub fn render(chooser: &Chooser, look: Look, body: Rect, buffer: &mut Buffer) -> Vec<(Aim, Rect)> {
    let Some(what) = chooser.choosing() else {
        return Vec::new();
    };
    let rows = rows(chooser);
    let footing: Vec<&str> = footing(what, !chooser.keys().is_empty())
        .split(" \u{b7} ")
        .collect();
    let caption = format!(" {} ", what.caption());

    let widest = rows
        .iter()
        .map(|row| row.chars().count())
        .chain(footing.iter().map(|line| line.chars().count() + 1))
        .chain(std::iter::once(caption.chars().count()))
        .max()
        .unwrap_or(0) as u16;
    let tall = (rows.len() + footing.len()) as u16 + BORDERS;
    let top = match what.asks_before_acting() {
        true => body.bottom().saturating_sub(tall + 1),
        false => body.y,
    };
    let area = placing::at(
        body,
        body.x.saturating_add(1),
        top,
        widest + LEAD + BORDERS + 1,
        tall,
    );
    if area.height <= BORDERS || area.width <= BORDERS {
        return Vec::new();
    }

    let inner = popup::frame::render(
        look,
        area,
        Line::styled(caption, look.palette.heading()),
        None,
        buffer,
    );
    let foot = (footing.len() as u16).min(inner.height.saturating_sub(1));
    let listed = Rect {
        height: inner.height - foot,
        ..inner
    };
    Paragraph::new(
        footing
            .iter()
            .take(foot as usize)
            .map(|line| Line::styled(format!(" {line}"), look.palette.quiet()))
            .collect::<Vec<Line<'static>>>(),
    )
    .render(
        Rect {
            y: listed.bottom(),
            height: foot,
            ..inner
        },
        buffer,
    );

    let mut aims = vec![(Aim::Popup, area)];
    aims.extend(
        popup::list::render(
            look,
            area,
            listed,
            rows.into_iter().map(Line::raw).collect(),
            chooser.at(),
            buffer,
        )
        .into_iter()
        .map(|(at, drawn)| (Aim::Option(at), drawn)),
    );
    aims
}

fn footing(what: crate::ui::Choosing, by_key: bool) -> &'static str {
    match (what.asks_before_acting(), by_key) {
        (true, true) => "a letter does it, on this host, now \u{b7} C or Esc walks away",
        (true, false) => {
            "\u{2191}\u{2193} choose \u{b7} Enter does it, on this host, now \u{b7} Esc leaves the host alone"
        }
        (false, _) => "\u{2191}\u{2193} choose \u{b7} Enter apply \u{b7} Esc leave it as it was",
    }
}

fn rows(chooser: &Chooser) -> Vec<String> {
    chooser
        .offered()
        .iter()
        .enumerate()
        .map(|(at, option)| match chooser.keys().get(at) {
            Some(key) => format!(" {key} {option}"),
            None => format!(" {option}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::helpers::words::text;
    use crate::ui::{Choosing, fixture};

    fn sorting(at: usize) -> Chooser {
        let mut chooser = Chooser::default();
        chooser.open(
            Choosing::Sort,
            vec![
                "as the agent sends it".into(),
                "TIME \u{2191}".into(),
                "TIME \u{2193}".into(),
                "SEVERITY \u{2191}".into(),
                "SEVERITY \u{2193}".into(),
            ],
            at,
        );
        chooser
    }

    fn drawn(chooser: &Chooser, width: u16, height: u16) -> (String, Vec<(Aim, Rect)>) {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
        let aims = render(chooser, fixture::look(), buffer.area, &mut buffer);
        (text::to_text(&buffer), aims)
    }

    #[test]
    fn the_option_a_reader_is_on_is_marked_with_an_arrow_and_not_only_a_colour() {
        let (page, _) = drawn(&sorting(1), 80, 24);

        assert!(page.contains("\u{25b8} TIME \u{2191}"), "{page}");
        assert!(page.contains("  TIME \u{2193}"), "{page}");
        assert!(
            !page.contains("\u{25b8} TIME \u{2193}"),
            "one arrow, on the one option Enter takes: {page}"
        );
    }

    #[test]
    fn the_options_stand_one_under_another_inside_a_rounded_frame() {
        let (page, _) = drawn(&sorting(0), 80, 24);
        let lines: Vec<&str> = page.lines().collect();

        let time = lines.iter().position(|line| line.contains("TIME \u{2191}"));
        let severity = lines
            .iter()
            .position(|line| line.contains("SEVERITY \u{2193}"));
        assert!(
            time.zip(severity)
                .is_some_and(|(time, severity)| severity == time + 3),
            "a list read down, not a band read across: {page}"
        );
        assert!(
            page.contains('\u{256d}') && page.contains('\u{256f}'),
            "{page}"
        );
        assert!(
            lines[0].contains("sort by"),
            "the caption sits in the frame: {page}"
        );
    }

    #[test]
    fn it_says_which_keys_move_it_which_applies_and_which_puts_it_back() {
        let (page, _) = drawn(&sorting(0), 80, 24);

        assert!(page.contains("Enter apply"), "{page}");
        assert!(page.contains("Esc leave it as it was"), "{page}");
    }

    #[test]
    fn every_option_is_on_the_screen_at_eighty_columns_and_the_popup_stays_on_it() {
        for (width, height) in [(80u16, 24u16), (100, 30), (40, 12)] {
            let (page, aims) = drawn(&sorting(4), width, height);
            for option in ["TIME \u{2191}", "SEVERITY \u{2193}"] {
                assert!(
                    page.contains(option),
                    "{width}: {option} is missing from {page}"
                );
            }
            for line in page.lines() {
                assert!(line.chars().count() <= width as usize, "{width}: {line}");
            }
            for (_, area) in aims {
                assert!(
                    area.right() <= width && area.bottom() <= height,
                    "{width}x{height}: {area:?}"
                );
            }
        }
    }

    #[test]
    fn each_option_drawn_is_known_by_the_place_it_was_drawn_in() {
        let (page, aims) = drawn(&sorting(0), 80, 24);
        let lines: Vec<&str> = page.lines().collect();

        let options: Vec<(Aim, Rect)> = aims
            .iter()
            .copied()
            .filter(|(aim, _)| matches!(aim, Aim::Option(_)))
            .collect();
        assert_eq!(options.len(), 5, "{aims:?}");
        let (aim, area) = options[3];
        assert_eq!(aim, Aim::Option(3));
        assert!(
            aims.first().is_some_and(
                |(aim, popup)| *aim == Aim::Popup && popup.contains(area.as_position())
            ),
            "the popup itself is known too, under its options, so a click beside them is still \
             a click inside it: {aims:?}"
        );
        assert!(
            lines[area.y as usize].contains("SEVERITY \u{2191}"),
            "the place recorded is the place the option is drawn: {page}"
        );
    }

    #[test]
    fn a_list_longer_than_the_room_scrolls_to_the_option_and_says_there_is_more() {
        let (page, aims) = drawn(&sorting(4), 80, 7);

        assert!(page.contains("\u{25b8} SEVERITY \u{2193}"), "{page}");
        assert!(
            aims.iter()
                .filter(|(aim, _)| matches!(aim, Aim::Option(_)))
                .count()
                < 5,
            "{aims:?}"
        );
        assert!(
            page.contains('\u{2588}'),
            "a scrollbar shows there is more: {page}"
        );
    }

    #[test]
    fn the_band_that_does_something_to_the_host_does_not_say_apply_like_the_others() {
        let mut chooser = Chooser::default();
        chooser.open_by_key(
            Choosing::Kill(vigil_model::KillTarget::Socket),
            vec![
                ('S', "ask the process to stop (SIGTERM)".into()),
                ('K', "stop the process now (SIGKILL)".into()),
            ],
        );
        let (page, _) = drawn(&chooser, 80, 12);
        assert!(
            !page
                .lines()
                .next()
                .unwrap_or_default()
                .contains("close them by"),
            "a popup that acts on the rows opens at the foot, off the top of the list where the \
             rows it acts on are: {page}"
        );

        assert!(page.contains("on this host, now"), "{page}");
        assert!(
            page.contains("C or Esc walks away"),
            "a reader looking for the way out of a destructive band must find it written \
             there, not remember it from the sort band: {page}"
        );
        assert!(
            page.contains("\u{25b8} S ask the process to stop (SIGTERM)"),
            "it opens on the gentlest of them, and each option carries the letter that \
             picks it: {page}"
        );
        assert!(page.contains("K stop the process now"), "{page}");
    }

    #[test]
    fn a_console_choosing_nothing_draws_no_popup_at_all() {
        let (page, aims) = drawn(&Chooser::default(), 80, 24);

        assert!(page.trim().is_empty(), "{page}");
        assert!(aims.is_empty());
    }
}
