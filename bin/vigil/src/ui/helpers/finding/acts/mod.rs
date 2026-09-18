use ratatui::text::{Line, Span};
use vigil_model::KillTarget;
use vigil_view::Offers;

use crate::ui::Look;
use crate::ui::helpers::layout::section;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Acts {
    pub acting: Option<KillTarget>,
    pub creating: bool,
    pub editing: bool,
    pub deleting: bool,
    pub control: bool,
    pub suppress: bool,
    pub history: bool,
    pub graph: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Button {
    pub key: char,
    pub name: &'static str,
}

impl Acts {
    #[cfg(test)]
    pub fn of_a_row(acting: Option<KillTarget>) -> Acts {
        Acts {
            acting,
            ..Acts::default()
        }
    }

    #[cfg(test)]
    pub fn historied(self, history: bool) -> Acts {
        Acts { history, ..self }
    }

    pub fn of_an_account(editing: bool, deleting: bool) -> Acts {
        Acts {
            editing,
            deleting,
            ..Acts::default()
        }
    }

    pub fn of_a_pane(offers: &Offers, on_a_row: bool) -> Acts {
        match on_a_row {
            false => Acts::default(),
            true => Acts {
                acting: offers.killing,
                creating: false,
                editing: false,
                deleting: false,
                control: offers.controlling.is_some(),
                suppress: offers.suppressing,
                history: offers.history,
                graph: offers.graph,
            },
        }
    }

    pub fn changing(self, creating: bool, editing: bool, deleting: bool) -> Acts {
        Acts {
            creating,
            editing,
            deleting,
            ..self
        }
    }

    pub fn buttons(self) -> Vec<Button> {
        let mut buttons = Vec::new();
        match self.acting {
            Some(target) => buttons.push(Button {
                key: 'K',
                name: match target {
                    KillTarget::Socket => "close this socket",
                    KillTarget::Program => "stop this program",
                },
            }),
            None => {
                if self.creating {
                    buttons.push(Button {
                        key: 'n',
                        name: "new",
                    });
                }
                if self.editing {
                    buttons.push(Button {
                        key: 'e',
                        name: "edit",
                    });
                }
                if self.deleting {
                    buttons.push(Button {
                        key: 'D',
                        name: "delete",
                    });
                }
            }
        }
        if self.control {
            buttons.push(Button {
                key: 'U',
                name: "start or stop it",
            });
        }
        if !buttons.is_empty() || self.suppress {
            buttons.push(Button {
                key: 'S',
                name: "suppress it",
            });
        }
        if self.graph {
            buttons.push(Button {
                key: 'P',
                name: "the path of a packet",
            });
        }
        if self.history {
            buttons.push(Button {
                key: 'H',
                name: "history",
            });
        }
        buttons
    }

    pub fn button(self, at: usize) -> Option<Button> {
        self.buttons().get(at).copied()
    }
}

pub type Place = (usize, usize, u16, u16);

const ARROWS: usize = 3;

pub fn laid(
    acts: Acts,
    at: Option<usize>,
    look: Look,
    width: usize,
) -> (Vec<Line<'static>>, Vec<Place>) {
    let buttons = acts.buttons();
    if buttons.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let arrows = match at {
        Some(_) => " \u{25b8} ",
        None => "   ",
    };
    let room = width.saturating_sub(4);
    let mut lines = vec![section::rule(look, "ACTIONS", width)];
    let mut places = Vec::new();
    let mut spans = vec![Span::raw(arrows)];
    let mut used = 0;

    for (index, button) in buttons.iter().enumerate() {
        let said = format!("{} {}", button.key, button.name);
        let drawn = match (at, at == Some(index)) {
            (Some(_), false) => format!("  {said}  "),
            _ => format!("[ {said} ]"),
        };
        let wanted = drawn.chars().count() + 1;
        if used > 0 && used + wanted > room {
            lines.push(Line::from(std::mem::take(&mut spans)));
            spans.push(Span::raw("   "));
            used = 0;
        }
        places.push((
            index,
            lines.len(),
            (ARROWS + used) as u16,
            (wanted - 1) as u16,
        ));
        used += wanted;
        spans.push(Span::styled(
            drawn,
            match (at, at == Some(index)) {
                (Some(_), true) => look.palette.selected(),
                (Some(_), false) => look.palette.quiet(),
                (None, _) => look.palette.accent(),
            },
        ));
        spans.push(Span::raw(" "));
    }
    lines.push(Line::from(spans));
    (lines, places)
}

#[cfg(test)]
mod tests;
