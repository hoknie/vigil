use ratatui::style::{Color, Modifier, Style};
use vigil_model::{CollectorState, Severity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    colour: bool,
}

impl Palette {
    pub fn from_environment() -> Self {
        Palette::decide(
            std::env::var("NO_COLOR").ok().as_deref(),
            std::env::var("TERM").ok().as_deref(),
        )
    }

    pub fn decide(no_color: Option<&str>, term: Option<&str>) -> Self {
        let refused = no_color.is_some_and(|value| !value.is_empty());
        let dumb = matches!(term, Some("dumb") | Some(""));
        Palette {
            colour: !refused && !dumb,
        }
    }

    pub fn heading(self) -> Style {
        self.tint(Style::default().add_modifier(Modifier::BOLD), Color::Cyan)
    }

    pub fn title(self) -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }

    pub fn quiet(self) -> Style {
        Style::default().add_modifier(Modifier::DIM)
    }

    pub fn border(self) -> Style {
        self.tint(Style::default().add_modifier(Modifier::DIM), Color::Blue)
    }

    pub fn label(self) -> Style {
        self.tint(Style::default(), Color::Cyan)
    }

    pub fn accent(self) -> Style {
        self.tint(Style::default(), Color::Cyan)
    }

    pub fn selected(self) -> Style {
        Style::default().add_modifier(Modifier::REVERSED)
    }

    pub fn marked(self) -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }

    pub fn alarm(self) -> Style {
        self.tint(Style::default().add_modifier(Modifier::BOLD), Color::Red)
    }

    pub fn healthy(self) -> Style {
        self.tint(Style::default(), Color::Green)
    }

    pub fn severity(self, severity: &Severity) -> Style {
        match (self.colour, severity) {
            (false, Severity::Critical | Severity::High) => {
                Style::default().add_modifier(Modifier::BOLD)
            }
            (false, _) => Style::default(),
            (true, Severity::Critical) => {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            }
            (true, Severity::High) => Style::default().fg(Color::Red),
            (true, Severity::Medium) => Style::default().fg(Color::Yellow),
            (true, Severity::Low) => Style::default().fg(Color::Cyan),
            (true, Severity::Info) => Style::default(),
            (true, Severity::Unknown(_)) => Style::default().fg(Color::Magenta),
        }
    }

    pub fn collector(self, state: &CollectorState) -> Style {
        match state {
            CollectorState::Ok => self.healthy(),
            CollectorState::Degraded => self.tint(Style::default(), Color::Yellow),
            CollectorState::Off => self.quiet(),
            _ => self.alarm(),
        }
    }

    fn tint(self, style: Style, colour: Color) -> Style {
        match self.colour {
            true => style.fg(colour),
            false => style,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_color_and_a_dumb_terminal_both_turn_colour_off() {
        assert!(!Palette::decide(Some("1"), Some("xterm-256color")).colour);
        assert!(!Palette::decide(None, Some("dumb")).colour);
        assert!(
            Palette::decide(Some(""), Some("xterm")).colour,
            "an empty NO_COLOR is not a refusal"
        );
        assert!(Palette::decide(None, Some("xterm-256color")).colour);
    }

    #[test]
    fn without_colour_nothing_anywhere_is_left_to_colour_alone() {
        let monochrome = Palette::decide(Some("1"), None);

        let mut styles = vec![
            monochrome.heading(),
            monochrome.title(),
            monochrome.quiet(),
            monochrome.border(),
            monochrome.accent(),
            monochrome.label(),
            monochrome.selected(),
            monochrome.marked(),
            monochrome.alarm(),
            monochrome.healthy(),
            monochrome.collector(&CollectorState::Ok),
            monochrome.collector(&CollectorState::Degraded),
            monochrome.collector(&CollectorState::Unavailable),
            monochrome.collector(&CollectorState::Off),
            monochrome.collector(&CollectorState::Unknown("odd".into())),
        ];
        styles.extend(Severity::KNOWN.iter().map(|it| monochrome.severity(it)));
        styles.push(monochrome.severity(&Severity::Unknown("odd".into())));

        for style in styles {
            assert_eq!(style.fg, None, "{style:?} would be told by colour alone");
            assert_eq!(style.bg, None, "{style:?} paints over the terminal's theme");
        }
    }

    #[test]
    fn a_collector_somebody_switched_off_is_not_drawn_as_a_broken_one() {
        let colour = Palette::decide(None, Some("xterm-256color"));

        assert_ne!(colour.collector(&CollectorState::Off), colour.alarm());
        assert_ne!(colour.collector(&CollectorState::Off), colour.healthy());
        assert_ne!(
            colour.collector(&CollectorState::Off),
            colour.collector(&CollectorState::Unavailable),
            "unavailable is an agent that cannot look; off is one that was told not to"
        );
        assert!(
            colour
                .collector(&CollectorState::Off)
                .add_modifier
                .contains(Modifier::DIM)
        );

        let monochrome = Palette::decide(Some("1"), None);
        assert_ne!(
            monochrome.collector(&CollectorState::Off),
            monochrome.collector(&CollectorState::Ok),
            "the difference must not be held by a colour that is not there"
        );
    }

    #[test]
    fn a_label_adds_nothing_at_all_when_there_is_no_colour_to_add() {
        let monochrome = Palette::decide(Some("1"), None);

        assert_eq!(monochrome.label(), Style::default());
        assert_ne!(
            Palette::decide(None, Some("xterm-256color")).label(),
            Style::default(),
            "and it does add something where there is colour to add"
        );
    }

    #[test]
    fn nothing_paints_its_own_background_except_the_cursor() {
        let colour = Palette::decide(None, Some("xterm-256color"));

        for style in [
            colour.heading(),
            colour.title(),
            colour.quiet(),
            colour.border(),
            colour.accent(),
            colour.label(),
            colour.alarm(),
            colour.healthy(),
            colour.severity(&Severity::Critical),
        ] {
            assert_eq!(style.bg, None, "{style:?}");
        }
        assert!(colour.selected().add_modifier.contains(Modifier::REVERSED));
    }
}
