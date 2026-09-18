use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use vigil_view::Pane;

use super::grouping;
use super::showing::Showing;
use crate::ui::{Arrows, Look, View};

const MARKER: u16 = 3;

const BETWEEN: u16 = 3;

const FOLDED: u16 = 2;

pub(in crate::ui) type Places = Vec<(usize, u16, u16)>;

pub(super) struct Menu {
    pub of_the_group: Vec<usize>,
    pub groups: Vec<(usize, Rect)>,
    pub names: Vec<(usize, Rect)>,
    pub said_nothing_to_show: bool,
}

pub(super) fn rows(groups: &[&'static str]) -> u16 {
    match groups.is_empty() {
        true => 1,
        false => 2,
    }
}

pub(super) fn render(
    look: Look,
    view: &View,
    showing: &Showing<'_>,
    panes: &[Box<dyn Pane>],
    groups: &[&'static str],
    over: Option<Rect>,
    buffer: &mut Buffer,
) -> Menu {
    let of_the_group = grouping::of_the_group(panes, &shown(view, panes), showing.group);
    let mut menu = Menu {
        of_the_group,
        groups: Vec::new(),
        names: Vec::new(),
        said_nothing_to_show: false,
    };
    let Some(over) = over else {
        return menu;
    };
    let lists = Rect {
        y: over.y + over.height - 1,
        height: 1,
        ..over
    };

    if !groups.is_empty() {
        let band = Rect { height: 1, ..over };
        let (line, places) = row(
            look,
            &grouping::named(groups),
            grouping::chosen(groups, showing.group),
            showing.arrows == Arrows::Groups,
            band.width,
        );
        Paragraph::new(line).render(band, buffer);
        menu.groups = put(places, band);
    }

    if menu.of_the_group.is_empty() {
        if let Some(notice) = grouping::nothing_to_show(view, panes, showing.group) {
            let said = notice.lines(look, lists.width as usize);
            Paragraph::new(said.into_iter().take(1).collect::<Vec<Line<'static>>>())
                .render(lists, buffer);
            menu.said_nothing_to_show = true;
        }
        return menu;
    }
    if menu.of_the_group.len() > 1 || !groups.is_empty() {
        let (line, places) = row(
            look,
            &named(panes, &menu.of_the_group),
            showing.at,
            showing.arrows == Arrows::Menu,
            lists.width,
        );
        Paragraph::new(line).render(lists, buffer);
        menu.names = put(places, lists);
    }

    menu
}

fn shown(view: &View, panes: &[Box<dyn Pane>]) -> Vec<usize> {
    (0..panes.len())
        .filter(|index| match view.reading(panes[*index].reads()) {
            crate::ui::Reading::Taken(reading) => panes[*index].shown(reading),
            _ => true,
        })
        .collect()
}

fn named(panes: &[Box<dyn Pane>], shown: &[usize]) -> Vec<(usize, String)> {
    shown
        .iter()
        .filter_map(|at| panes.get(*at).map(|pane| (*at, pane.name().to_string())))
        .collect()
}

fn put(places: Places, over: Rect) -> Vec<(usize, Rect)> {
    places
        .into_iter()
        .map(|(at, x, wide)| {
            (
                at,
                Rect::new(over.x + x, over.y, wide, 1).intersection(over),
            )
        })
        .collect()
}

pub(in crate::ui) fn row(
    look: Look,
    named: &[(usize, String)],
    at: usize,
    focused: bool,
    room: u16,
) -> (Line<'static>, Places) {
    match spread(named) > room {
        true => folded(look, named, at, focused),
        false => beside_each_other(look, named, at, focused),
    }
}

fn spread(named: &[(usize, String)]) -> u16 {
    let names: u16 = named
        .iter()
        .map(|(_, name)| name.chars().count() as u16 + FOLDED)
        .sum();

    MARKER + names + BETWEEN * (named.len().saturating_sub(1) as u16)
}

fn beside_each_other(
    look: Look,
    named: &[(usize, String)],
    at: usize,
    focused: bool,
) -> (Line<'static>, Places) {
    let mut places = Places::new();
    let mut x = MARKER;
    let mut spans = vec![marker(look, focused)];

    for (place, (index, name)) in named.iter().enumerate() {
        if place > 0 {
            spans.push(Span::styled(" · ", look.palette.border()));
            x += BETWEEN;
        }
        let wide = name.chars().count() as u16 + FOLDED;
        places.push((*index, x, wide));
        x += wide;
        match *index == at {
            true => spans.push(Span::styled(
                format!("[{name}]"),
                match focused {
                    true => look.palette.selected(),
                    false => look.palette.heading(),
                },
            )),
            false => spans.push(Span::raw(format!(" {name} "))),
        }
    }

    (Line::from(spans), places)
}

fn folded(
    look: Look,
    named: &[(usize, String)],
    at: usize,
    focused: bool,
) -> (Line<'static>, Places) {
    let here = named
        .iter()
        .position(|(index, _)| *index == at)
        .unwrap_or(0);
    let count = named.len().max(1);
    let (before, _) = named[(here + count - 1) % count];
    let (after, _) = named[(here + 1) % count];
    let name = named
        .get(here)
        .map(|(_, name)| name.clone())
        .unwrap_or_default();
    let wide = name.chars().count() as u16 + FOLDED;

    (
        Line::from(vec![
            marker(look, focused),
            Span::styled("< ", look.palette.border()),
            Span::styled(
                format!("[{name}]"),
                match focused {
                    true => look.palette.selected(),
                    false => look.palette.heading(),
                },
            ),
            Span::styled(" >", look.palette.border()),
        ]),
        vec![
            (before, MARKER, FOLDED),
            (at, MARKER + FOLDED, wide),
            (after, MARKER + FOLDED + wide, FOLDED),
        ],
    )
}

fn marker(look: Look, focused: bool) -> Span<'static> {
    Span::styled(
        match focused {
            true => " \u{25b8} ",
            false => "   ",
        },
        look.palette.heading(),
    )
}
