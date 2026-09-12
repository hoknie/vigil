use crate::ui::Choice;
use crate::ui::helpers::motion::step_along::step_along;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum System {
    #[default]
    Host,
    Files,
}

impl System {
    pub const ALL: &'static [System] = &[System::Host, System::Files];

    pub fn name(self) -> &'static str {
        match self {
            System::Host => "the host",
            System::Files => "watched files",
        }
    }

    pub fn caption(self) -> &'static str {
        match self {
            System::Host => "THE HOST",
            System::Files => "WATCHED FILES",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            System::Host => "THE SELECTED PART",
            System::Files => "THE SELECTED PATH",
        }
    }

    pub fn collector(self) -> &'static str {
        match self {
            System::Host => "resources",
            System::Files => "files",
        }
    }

    pub fn holding(key: &str) -> System {
        match key.starts_with("file|") || key.starts_with("directory|") {
            true => System::Files,
            false => System::Host,
        }
    }
}

impl Choice for System {
    const COUNT: usize = System::ALL.len();

    fn index(self) -> usize {
        System::ALL
            .iter()
            .position(|half| *half == self)
            .unwrap_or(0)
    }

    fn step(self, by: isize, shown: &[System]) -> System {
        step_along(self, by, shown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_readings_of_this_section_are_named_on_the_screen_rather_than_guessed_at() {
        assert_eq!(System::ALL.len(), 2);
        for half in System::ALL {
            assert!(!half.name().is_empty());
            assert!(!half.caption().is_empty());
        }
        assert_ne!(System::Host.collector(), System::Files.collector());
    }

    #[test]
    fn a_row_of_either_reading_is_shown_in_the_view_that_draws_it() {
        assert_eq!(System::holding("file|/etc/hosts"), System::Files);
        assert_eq!(System::holding("directory|/usr/bin"), System::Files);
        assert_eq!(System::holding("fs|/var"), System::Host);
        assert_eq!(System::holding("boot|current"), System::Host);
        assert_eq!(System::holding("memory|summary"), System::Host);
    }

    #[test]
    fn the_host_is_the_view_a_reader_opening_the_section_is_given() {
        assert_eq!(System::default(), System::Host);
    }
}
