use ratatui::text::{Line, Span};

use crate::ui::Look;

pub const ALL: &[(char, &str)] = &[
    ('t', "tcp"),
    ('T', "tcp6"),
    ('u', "udp"),
    ('U', "udp6"),
    ('x', "unix"),
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Protocols {
    hidden: Vec<String>,
}

impl Protocols {
    pub fn showing(&self, protocol: &str) -> bool {
        !self.hidden.iter().any(|name| name == protocol)
    }

    pub fn holding_back(&self) -> bool {
        !self.hidden.is_empty()
    }

    pub fn toggle(&mut self, key: char) -> bool {
        let Some((_, name)) = ALL.iter().find(|(letter, _)| *letter == key) else {
            return false;
        };
        match self.hidden.iter().position(|hidden| hidden == name) {
            Some(at) => {
                self.hidden.remove(at);
            }
            None => self.hidden.push((*name).to_string()),
        }
        true
    }

    pub fn clear(&mut self) {
        self.hidden.clear();
    }

    pub fn describe(&self) -> String {
        match self.hidden.is_empty() {
            true => "every kind of socket".to_string(),
            false => format!("hiding {}", self.hidden.join(", ")),
        }
    }

    pub fn line(&self, look: Look) -> Line<'static> {
        let mut spans = vec![Span::styled(" kinds ", look.palette.heading())];
        for (key, name) in ALL {
            let label = match self.showing(name) {
                true => format!("[{key} {name}]"),
                false => format!(" {key} {name} "),
            };
            spans.push(Span::styled(
                label,
                match self.showing(name) {
                    true => look.palette.selected(),
                    false => look.palette.quiet(),
                },
            ));
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled("· a all", look.palette.quiet()));
        Line::from(spans)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::{Paragraph, Widget};

    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn drawn(protocols: &Protocols, width: u16) -> String {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, 1));
        Paragraph::new(protocols.line(fixture::look())).render(buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn a_fresh_screen_shows_every_kind_of_socket() {
        let protocols = Protocols::default();

        for (_, name) in ALL {
            assert!(protocols.showing(name), "{name}");
        }
        assert!(!protocols.holding_back());
    }

    #[test]
    fn several_can_be_off_at_once_which_is_the_whole_point() {
        let mut protocols = Protocols::default();

        assert!(protocols.toggle('x'));
        assert!(protocols.toggle('U'));

        assert!(protocols.showing("tcp"));
        assert!(protocols.showing("tcp6"));
        assert!(protocols.showing("udp"));
        assert!(!protocols.showing("udp6"));
        assert!(!protocols.showing("unix"));
        assert!(protocols.holding_back());
    }

    #[test]
    fn the_same_key_puts_it_back_and_a_key_that_is_not_one_of_them_is_not_swallowed() {
        let mut protocols = Protocols::default();
        protocols.toggle('t');
        assert!(!protocols.showing("tcp"));
        protocols.toggle('t');
        assert!(protocols.showing("tcp"));

        assert!(
            !protocols.toggle('z'),
            "a key this filter does not own has to reach whatever else wanted it"
        );
    }

    #[test]
    fn a_protocol_this_build_has_never_heard_of_is_shown_rather_than_hidden() {
        let mut protocols = Protocols::default();
        protocols.toggle('t');

        assert!(protocols.showing("sctp"));
    }

    #[test]
    fn what_is_showing_is_readable_with_no_colour_at_all() {
        let mut protocols = Protocols::default();
        protocols.toggle('u');

        let line = drawn(&protocols, 80);

        assert!(line.contains("[t tcp]"), "{line}");
        assert!(line.contains(" u udp "), "{line}");
        assert!(!line.contains("[u udp]"), "{line}");
        assert!(
            line.contains("a all"),
            "the way back is on the line too: {line}"
        );
    }

    #[test]
    fn it_fits_the_narrowest_terminal_this_is_read_on() {
        for width in [80u16, 120, 200] {
            let line = drawn(&Protocols::default(), width);
            assert!(line.chars().count() <= width as usize, "{width}: {line}");
        }
    }
}
