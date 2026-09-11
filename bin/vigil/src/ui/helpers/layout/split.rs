use ratatui::layout::Rect;

const LIST_FLOOR: u16 = 3 + 1 + 8 + 1 + 8 + 1 + 24;

const PANEL_FLOOR: u16 = 3 + 2 + 24 + 1 + 24;

const RULE: u16 = 3;

const LIST_SHARE: u16 = 45;

const SPLIT_AT: u16 = narrowest();

const _: () = assert!(SPLIT_AT > 80);

const fn narrowest() -> u16 {
    let mut width = LIST_FLOOR + RULE + PANEL_FLOOR;
    while width < 512 {
        let across = (width - RULE) as u32;
        let list = across * LIST_SHARE as u32 / 100;
        if list >= LIST_FLOOR as u32 && across - list >= PANEL_FLOOR as u32 {
            return width;
        }
        width += 1;
    }
    width
}

pub struct Split {
    pub list: Rect,
    pub rule: Rect,
    pub panel: Rect,
}

pub struct Layout {
    pub list: Option<Rect>,
    pub list_caption: Option<Rect>,
    pub rule: Option<Rect>,
    pub panel: Option<Rect>,
    pub detail_caption: Option<Rect>,
}

impl Layout {
    pub fn detail(&self) -> Option<Rect> {
        self.panel
    }
}

pub fn layout(body: Rect, open: bool) -> Layout {
    let empty = Layout {
        list: None,
        list_caption: None,
        rule: None,
        panel: None,
        detail_caption: None,
    };

    if !open {
        return Layout {
            list: Some(body),
            ..empty
        };
    }

    match beside(body) {
        None => Layout {
            detail_caption: Some(first_line(body)),
            panel: Some(under_the_first_line(body)),
            ..empty
        },
        Some(split) => Layout {
            list_caption: Some(first_line(split.list)),
            list: Some(under_the_first_line(split.list)),
            rule: Some(split.rule),
            detail_caption: Some(first_line(split.panel)),
            panel: Some(under_the_first_line(split.panel)),
        },
    }
}

fn first_line(area: Rect) -> Rect {
    Rect { height: 1, ..area }
}

fn under_the_first_line(area: Rect) -> Rect {
    Rect {
        y: area.y + 1,
        height: area.height.saturating_sub(1),
        ..area
    }
}

pub fn beside(area: Rect) -> Option<Split> {
    if area.width < SPLIT_AT {
        return None;
    }

    let across = area.width - RULE;
    let list = (u32::from(across) * u32::from(LIST_SHARE) / 100) as u16;

    Some(Split {
        list: Rect {
            width: list,
            ..area
        },
        rule: Rect {
            x: area.x + list,
            width: RULE,
            ..area
        },
        panel: Rect {
            x: area.x + list + RULE,
            width: across - list,
            ..area
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_terminal_with_no_room_for_both_halves_gets_one_of_them_whole() {
        assert!(beside(Rect::new(0, 0, 80, 24)).is_none());
        assert!(beside(Rect::new(0, 0, SPLIT_AT - 1, 24)).is_none());
        assert!(beside(Rect::new(0, 0, SPLIT_AT, 24)).is_some());
        assert!(beside(Rect::new(0, 0, 200, 24)).is_some());
    }

    #[test]
    fn wherever_it_does_divide_both_halves_clear_their_own_floor() {
        for width in SPLIT_AT..=400 {
            let split = beside(Rect::new(0, 0, width, 24)).expect("wide enough to divide");

            assert!(
                split.list.width >= LIST_FLOOR,
                "{width} columns leaves the list {}",
                split.list.width
            );
            assert!(
                split.panel.width >= PANEL_FLOOR,
                "{width} columns leaves the panel {}",
                split.panel.width
            );
        }
    }

    #[test]
    fn one_column_narrower_does_not_fit_so_the_threshold_is_not_generous() {
        let across = SPLIT_AT - 1 - RULE;
        let list = (u32::from(across) * u32::from(LIST_SHARE) / 100) as u16;

        assert!(
            list < LIST_FLOOR || across - list < PANEL_FLOOR,
            "one column narrower still fits, so the threshold is higher than it needs to be"
        );
    }

    #[test]
    fn the_number_it_comes_out_at_is_the_one_the_screens_were_looked_at() {
        assert_eq!(SPLIT_AT, 106);
    }

    #[test]
    fn the_captions_take_one_line_each_and_the_panes_get_the_rest() {
        let body = Rect::new(0, 0, 200, 30);
        let laid_out = layout(body, true);

        let caption = laid_out.detail_caption.expect("a caption over the panel");
        let panel = laid_out.detail().expect("a panel");
        assert_eq!(caption.height, 1);
        assert_eq!(panel.y, caption.y + 1);
        assert_eq!(panel.height, body.height - 1);
        assert_eq!(panel.width, caption.width);
    }

    #[test]
    fn a_screen_with_one_pane_on_it_still_says_how_far_down_it_the_reader_is() {
        let body = Rect::new(0, 0, 80, 24);

        let laid_out = layout(body, true);

        assert!(laid_out.detail_caption.is_some());
        assert_eq!(laid_out.detail().expect("a panel").height, body.height - 1);
        assert!(
            laid_out.list.is_none(),
            "there is no room for the list as well"
        );
    }

    #[test]
    fn with_the_panel_closed_the_list_has_the_whole_body_and_no_caption() {
        let body = Rect::new(0, 0, 200, 30);

        let laid_out = layout(body, false);

        assert_eq!(laid_out.list, Some(body));
        assert!(laid_out.list_caption.is_none());
        assert!(laid_out.detail().is_none());
    }

    #[test]
    fn the_three_pieces_tile_the_body_exactly_with_nothing_lost_between_them() {
        let area = Rect::new(4, 2, 200, 30);
        let split = beside(area).expect("wide enough");

        assert_eq!(split.list.x, area.x);
        assert_eq!(split.rule.x, split.list.right());
        assert_eq!(split.panel.x, split.rule.right());
        assert_eq!(split.panel.right(), area.right());
        assert_eq!(split.list.height, area.height);
        assert_eq!(split.panel.height, area.height);
    }
}
