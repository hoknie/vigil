use super::group::Group;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Home,
    Ports,
    Accounts,
    Programs,
    Startup,
    Firewall,
    Summary,
    Findings,
}

const HOME: &str = "home";

const NUMBERED: usize = 9;

impl Screen {
    pub const ALL: &'static [Screen] = &[
        Screen::Ports,
        Screen::Accounts,
        Screen::Programs,
        Screen::Startup,
        Screen::Firewall,
        Screen::Summary,
        Screen::Findings,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Screen::Home => HOME,
            Screen::Ports => "ports",
            Screen::Accounts => "accounts",
            Screen::Programs => "programs",
            Screen::Startup => "startup",
            Screen::Firewall => "firewall",
            Screen::Summary => "summary",
            Screen::Findings => "findings",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Screen::Home => "What this agent watches",
            Screen::Ports => "What is listening",
            Screen::Accounts => "Who can log in",
            Screen::Programs => "What has run here",
            Screen::Startup => "What starts by itself",
            Screen::Firewall => "What the host lets in",
            Screen::Summary => "This host and its agent",
            Screen::Findings => "What the agent has found",
        }
    }

    pub fn holds(self) -> &'static str {
        match self {
            Screen::Home => "every section of this console",
            Screen::Ports => "what is listening",
            Screen::Accounts => "who can log in",
            Screen::Programs => "what has run here",
            Screen::Startup => "what starts by itself",
            Screen::Firewall => "what the host lets in",
            Screen::Summary => "this host and its agent",
            Screen::Findings => "what it has found",
        }
    }

    pub fn group(self) -> Group {
        match self {
            Screen::Ports
            | Screen::Accounts
            | Screen::Programs
            | Screen::Startup
            | Screen::Firewall => Group::Reads,
            Screen::Home | Screen::Summary | Screen::Findings => Group::Concludes,
        }
    }

    pub fn collectors(self) -> &'static [&'static str] {
        match self {
            Screen::Ports => &["ports"],
            Screen::Accounts => &["users"],
            Screen::Programs => &["processes", "launches"],
            Screen::Startup => &["persistence"],
            Screen::Firewall => &["firewall"],
            Screen::Home | Screen::Summary | Screen::Findings => &[],
        }
    }

    pub fn holding(collector: &str) -> Option<Screen> {
        Screen::ALL
            .iter()
            .copied()
            .find(|screen| screen.collectors().contains(&collector))
    }

    pub fn parse(name: &str) -> Option<Screen> {
        if name == HOME {
            return Some(Screen::Home);
        }
        Screen::ALL
            .iter()
            .copied()
            .find(|screen| screen.name() == name)
    }

    pub fn digit(self) -> Option<u8> {
        let at = Screen::ALL.iter().position(|screen| *screen == self)?;
        match at < NUMBERED {
            true => Some(at as u8 + 1),
            false => None,
        }
    }

    pub fn numbered() -> usize {
        Screen::ALL.len().min(NUMBERED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_screen_has_a_name_a_script_can_ask_for() {
        for screen in Screen::ALL {
            assert_eq!(Screen::parse(screen.name()), Some(*screen));
        }
        assert_eq!(
            Screen::parse("home"),
            Some(Screen::Home),
            "the main screen is asked for by name too, and is not one of the numbered sections"
        );
        assert_eq!(Screen::parse("resources"), None);
    }

    #[test]
    fn renumbering_the_sections_is_a_deliberate_edit_and_not_a_side_effect() {
        let pairs: Vec<(Option<u8>, &str)> = Screen::ALL
            .iter()
            .map(|screen| (screen.digit(), screen.name()))
            .collect();

        assert_eq!(
            pairs,
            vec![
                (Some(1), "ports"),
                (Some(2), "accounts"),
                (Some(3), "programs"),
                (Some(4), "startup"),
                (Some(5), "firewall"),
                (Some(6), "summary"),
                (Some(7), "findings"),
            ],
            "a new section moved the numbers of the sections below it: rewrite this table by \
             hand, and the help and the pty run with it"
        );
    }

    #[test]
    fn what_the_agent_reads_is_numbered_before_what_it_makes_of_the_reading() {
        let reads: Vec<u8> = Screen::ALL
            .iter()
            .filter(|screen| screen.group() == Group::Reads)
            .filter_map(|screen| screen.digit())
            .collect();
        let concludes: Vec<u8> = Screen::ALL
            .iter()
            .filter(|screen| screen.group() == Group::Concludes)
            .filter_map(|screen| screen.digit())
            .collect();

        assert!(
            reads.iter().max() < concludes.iter().min(),
            "the numbers are positions in ALL and the main screen draws the two groups in \
             order: a reading numbered after a conclusion puts the row of one group between \
             the rows of the other. Reads {reads:?}, concludes {concludes:?}"
        );
    }

    #[test]
    fn the_main_screen_has_no_number_because_escape_is_the_way_back_to_it() {
        assert_eq!(Screen::Home.digit(), None);
        assert!(!Screen::ALL.contains(&Screen::Home));
    }

    #[test]
    fn a_section_past_the_ninth_draws_no_number_and_is_opened_by_the_cursor() {
        assert!(
            Screen::numbered() <= 9,
            "there are nine digits, and the tenth section is reached with the cursor"
        );
        assert_eq!(
            Screen::ALL
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
        for screen in Screen::ALL.iter().chain([Screen::Home].iter()) {
            assert!(!screen.title().is_empty(), "{}", screen.name());
            assert!(!screen.holds().is_empty(), "{}", screen.name());
        }
    }

    #[test]
    fn every_collector_a_section_reads_belongs_to_that_one_section_and_no_other() {
        for screen in Screen::ALL {
            for collector in screen.collectors() {
                assert_eq!(
                    Screen::holding(collector),
                    Some(*screen),
                    "{collector} is claimed by two sections"
                );
            }
        }
        assert_eq!(Screen::holding("resources"), None);
    }
}
