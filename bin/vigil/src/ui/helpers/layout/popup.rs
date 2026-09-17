use ratatui::layout::Rect;

const SHADOW: u16 = 1;

pub fn at(bound: Rect, x: u16, y: u16, width: u16, height: u16) -> Rect {
    let width = width.min(bound.width.saturating_sub(SHADOW));
    let height = height.min(bound.height.saturating_sub(SHADOW));
    let right = bound.right().saturating_sub(SHADOW);
    let bottom = bound.bottom().saturating_sub(SHADOW);
    Rect {
        x: x.clamp(bound.x, right.saturating_sub(width).max(bound.x)),
        y: y.clamp(bound.y, bottom.saturating_sub(height).max(bound.y)),
        width,
        height,
    }
}

pub fn beside(bound: Rect, x: u16, line: u16, width: u16, height: u16) -> Rect {
    let below = bound.bottom().saturating_sub(line + 1 + SHADOW);
    let above = line.saturating_sub(bound.y + SHADOW);
    match height > below && above > below {
        true => {
            let height = height.min(above);
            at(bound, x, line - SHADOW - height, width, height)
        }
        false => at(bound, x, line + 1, width, height.min(below)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Rect = Rect::new(0, 0, 80, 24);

    fn inside(popup: Rect) -> bool {
        popup.right() < SCREEN.right() && popup.bottom() < SCREEN.bottom()
    }

    #[test]
    fn a_popup_wider_or_taller_than_the_screen_is_cut_to_it_with_room_left_for_its_shadow() {
        let popup = at(SCREEN, 70, 20, 200, 90);

        assert!(inside(popup), "{popup:?}");
        assert_eq!((popup.width, popup.height), (79, 23));
    }

    #[test]
    fn a_popup_asked_for_near_the_edge_is_moved_back_onto_the_screen_rather_than_cut() {
        let popup = at(SCREEN, 75, 22, 20, 6);

        assert!(inside(popup), "{popup:?}");
        assert_eq!((popup.width, popup.height), (20, 6));
    }

    #[test]
    fn a_list_opens_under_its_field_and_over_it_only_when_below_is_the_smaller_room() {
        let under = beside(SCREEN, 12, 5, 30, 8);
        assert_eq!(under.y, 6, "{under:?}");

        let over = beside(SCREEN, 12, 20, 30, 8);
        assert_eq!(
            over.bottom() + SHADOW,
            20,
            "the list and its shadow end on the line above the field: {over:?}"
        );
        assert!(inside(over), "{over:?}");

        let squeezed = beside(SCREEN, 12, 12, 30, 40);
        assert!(inside(squeezed), "{squeezed:?}");
        assert!(
            squeezed.y > 12 || squeezed.bottom() + SHADOW <= 12,
            "neither the list nor its shadow covers its own field: {squeezed:?}"
        );
    }
}
