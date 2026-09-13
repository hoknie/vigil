use ratatui::layout::Rect;

use crate::ui::Look;

const ROOM_FOR_THE_DEFINITION: u16 = 7;

pub fn about(area: Rect, look: Look, lines: u16) -> (Option<Rect>, Rect) {
    let short = look.interactive() && area.height < ROOM_FOR_THE_DEFINITION + lines;
    if lines == 0 || short {
        return (None, area);
    }
    (
        Some(Rect {
            height: lines,
            ..area
        }),
        below(area, lines),
    )
}

fn below(area: Rect, lines: u16) -> Rect {
    Rect {
        y: area.y + lines,
        height: area.height.saturating_sub(lines),
        ..area
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::fixture;

    #[test]
    fn a_band_taken_off_the_top_leaves_the_rest_of_the_area_and_loses_no_row() {
        let area = Rect::new(0, 0, 80, 20);

        let (band, rest) = about(area, fixture::look(), 2);

        let band = band.expect("room for a sentence about the list");
        assert_eq!(band.height, 2);
        assert_eq!(rest.y, area.y + 2);
        assert_eq!(band.height + rest.height, area.height);
    }

    #[test]
    fn a_terminal_with_no_room_keeps_the_table_rather_than_the_decoration() {
        assert!(
            about(Rect::new(0, 0, 80, 6), fixture::look(), 2)
                .0
                .is_none()
        );
    }
}
