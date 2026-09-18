use super::group::Group;
use crate::ui::screens::unknown;
use crate::ui::{holding, sections};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Screen {
    name: &'static str,
}

const NUMBERED: usize = 9;

const OF_THE_CONSOLE: &[(&str, &str, &str, Group)] = &[
    (
        "home",
        "What this agent watches",
        "every section of this console",
        Group::Concludes,
    ),
    (
        "summary",
        "This host and its agent",
        "this host and its agent",
        Group::Concludes,
    ),
    (
        "findings",
        "What the agent has found",
        "what it has found",
        Group::Concludes,
    ),
    ("unknown", unknown::TITLE, unknown::HOLDS, Group::Reads),
];

impl Default for Screen {
    fn default() -> Screen {
        Screen::HOME
    }
}

const FORMER_NAME_OF_THE_NETWORK: &str = "ports";

impl Screen {
    pub const HOME: Screen = Screen { name: "home" };

    pub const SUMMARY: Screen = Screen { name: "summary" };

    pub const FINDINGS: Screen = Screen { name: "findings" };

    pub const UNKNOWN: Screen = Screen { name: "unknown" };

    pub fn all() -> Vec<Screen> {
        let mut every: Vec<Screen> = sections()
            .iter()
            .map(|section| Screen {
                name: section.name(),
            })
            .collect();
        every.push(Screen::SUMMARY);
        every.push(Screen::FINDINGS);
        every
    }

    pub fn name(self) -> &'static str {
        self.name
    }

    pub fn title(self) -> &'static str {
        match self.of_the_console() {
            Some((title, _, _)) => title,
            None => holding(self.name).map_or("", |section| section.title()),
        }
    }

    pub fn holds(self) -> &'static str {
        match self.of_the_console() {
            Some((_, holds, _)) => holds,
            None => holding(self.name).map_or("", |section| section.holds()),
        }
    }

    pub fn group(self) -> Group {
        match self.of_the_console() {
            Some((_, _, group)) => group,
            None => Group::Reads,
        }
    }

    pub fn collectors(self) -> Vec<String> {
        let Some(section) = holding(self.name) else {
            return Vec::new();
        };

        let mut named: Vec<String> = Vec::new();
        for pane in section.panes() {
            if !named.iter().any(|already| already == pane.reads()) {
                named.push(pane.reads().to_string());
            }
        }
        named
    }

    pub fn draws_a_reading(self) -> bool {
        self.group() == Group::Reads
    }

    pub fn shows_what_has_gone(self) -> bool {
        holding(self.name).is_some_and(|section| section.shows_what_has_gone())
    }

    pub fn showing(collector: &str) -> Option<Screen> {
        Screen::all()
            .into_iter()
            .find(|screen| screen.collectors().iter().any(|named| named == collector))
    }

    pub fn parse(name: &str) -> Option<Screen> {
        if let Some((of_the_console, _, _, _)) =
            OF_THE_CONSOLE.iter().find(|(named, ..)| *named == name)
        {
            return Some(Screen {
                name: of_the_console,
            });
        }
        let name = match name {
            FORMER_NAME_OF_THE_NETWORK => "network",
            other => other,
        };
        Screen::all()
            .into_iter()
            .find(|screen| screen.name() == name)
    }

    pub fn digit(self) -> Option<u8> {
        let at = Screen::all().iter().position(|screen| *screen == self)?;

        match at < NUMBERED {
            true => u8::try_from(at + 1).ok(),
            false => None,
        }
    }

    pub fn numbered() -> usize {
        Screen::all()
            .iter()
            .filter(|screen| screen.digit().is_some())
            .count()
            .min(NUMBERED)
    }

    pub fn unnumbered() -> Vec<Screen> {
        Screen::all()
            .into_iter()
            .filter(|screen| screen.digit().is_none())
            .collect()
    }

    fn of_the_console(self) -> Option<(&'static str, &'static str, Group)> {
        OF_THE_CONSOLE
            .iter()
            .find(|(name, ..)| *name == self.name)
            .map(|(_, title, holds, group)| (*title, *holds, *group))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_screen_has_a_name_a_script_can_ask_for() {
        for screen in Screen::all() {
            assert_eq!(Screen::parse(screen.name()), Some(screen));
        }
        assert_eq!(
            Screen::parse("home"),
            Some(Screen::HOME),
            "the main screen is asked for by name too, and is not one of the numbered sections"
        );
        assert_eq!(Screen::parse("resources"), None);
    }

    #[test]
    fn renumbering_the_sections_is_a_deliberate_edit_and_not_a_side_effect() {
        let pairs: Vec<(Option<u8>, &str)> = Screen::all()
            .iter()
            .map(|screen| (screen.digit(), screen.name()))
            .collect();

        assert_eq!(
            pairs,
            vec![
                (Some(1), "network"),
                (Some(2), "accounts"),
                (Some(3), "programs"),
                (Some(4), "startup"),
                (Some(5), "firewall"),
                (Some(6), "system"),
                (Some(7), "containers"),
                (Some(8), "summary"),
                (Some(9), "findings"),
            ],
            "the numbers are the order the modules are listed in: moving a module in the \
             registry moves the key that opens it, and the help and the pty run with it"
        );
    }

    #[test]
    fn what_the_agent_reads_is_numbered_before_what_it_makes_of_the_reading() {
        let reads: Vec<u8> = Screen::all()
            .iter()
            .filter(|screen| screen.group() == Group::Reads)
            .filter_map(|screen| screen.digit())
            .collect();
        let concludes: Vec<u8> = Screen::all()
            .iter()
            .filter(|screen| screen.group() == Group::Concludes)
            .filter_map(|screen| screen.digit())
            .collect();

        assert!(
            reads.iter().max() < concludes.iter().min(),
            "the numbers are positions in the list and the main screen draws the two groups \
             in order: a reading numbered after a conclusion puts the row of one group between \
             the rows of the other. Reads {reads:?}, concludes {concludes:?}"
        );
    }

    #[test]
    fn the_main_screen_has_no_number_because_escape_is_the_way_back_to_it() {
        assert_eq!(Screen::HOME.digit(), None);
        assert!(!Screen::all().contains(&Screen::HOME));
    }

    #[test]
    fn a_section_past_the_ninth_draws_no_number_and_is_opened_by_the_cursor() {
        assert!(
            Screen::numbered() <= 9,
            "there are nine digits, and the tenth section is reached with the cursor"
        );
        assert_eq!(
            Screen::all()
                .iter()
                .filter(|screen| screen.digit().is_some())
                .count(),
            Screen::numbered()
        );
    }

    #[test]
    fn the_difference_is_not_a_screen_and_cannot_be_asked_for_as_one() {
        assert_eq!(Screen::parse("difference"), None);
    }

    #[test]
    fn every_screen_says_what_looking_at_it_is_for() {
        for screen in Screen::all().iter().chain([Screen::HOME].iter()) {
            assert!(!screen.title().is_empty(), "{}", screen.name());
            assert!(!screen.holds().is_empty(), "{}", screen.name());
        }
    }

    #[test]
    fn every_collector_a_section_reads_belongs_to_that_one_section_and_no_other() {
        for screen in Screen::all() {
            for collector in screen.collectors() {
                assert_eq!(
                    Screen::showing(&collector),
                    Some(screen),
                    "{collector} is claimed by two sections"
                );
            }
        }
        assert_eq!(
            Screen::showing("resources"),
            Some(Screen::parse("system").expect("the host and its files")),
            "one section holds two readings, and both of them belong to it"
        );
        assert_eq!(Screen::showing("nothing-of-the-sort"), None);
    }

    #[test]
    fn the_three_screens_the_console_writes_itself_are_the_ones_no_module_declares() {
        for screen in [Screen::HOME, Screen::SUMMARY, Screen::FINDINGS] {
            assert!(
                holding(screen.name()).is_none(),
                "{} is declared by a module and written out here as well",
                screen.name()
            );
            assert_eq!(screen.group(), Group::Concludes);
            assert!(screen.collectors().is_empty());
        }
        for screen in Screen::all() {
            if screen.group() == Group::Reads {
                assert!(
                    holding(screen.name()).is_some(),
                    "{} is drawn from a table of its own rather than from the module that \
                     declares it",
                    screen.name()
                );
            }
        }
    }
}
