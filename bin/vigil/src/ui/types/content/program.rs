use crate::ui::Choice;
use crate::ui::helpers::motion::step_along::step_along;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Program {
    #[default]
    Running,
    Launches,
}

impl Program {
    pub const ALL: &'static [Program] = &[Program::Running, Program::Launches];

    pub fn name(self) -> &'static str {
        match self {
            Program::Running => "running",
            Program::Launches => "launches",
        }
    }

    pub fn caption(self) -> &'static str {
        match self {
            Program::Running => "RUNNING",
            Program::Launches => "LAUNCHES",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Program::Running => "THE SELECTED PROGRAM",
            Program::Launches => "THE SELECTED LAUNCH",
        }
    }

    pub fn about(self) -> &'static str {
        match self {
            Program::Running => "one row per program found running, and as whom",
            Program::Launches => {
                "one row per person and program the kernel's audit records have seen run; \
                 rows are never removed from it"
            }
        }
    }

    pub fn collector(self) -> &'static str {
        match self {
            Program::Running => "processes",
            Program::Launches => "launches",
        }
    }

    pub fn things(self, how_many: usize) -> &'static str {
        match (self, how_many) {
            (Program::Running, 1) => "program",
            (Program::Running, _) => "programs",
            (Program::Launches, 1) => "launch",
            (Program::Launches, _) => "launches",
        }
    }

    pub fn holding(key: &str) -> Program {
        match key.starts_with("exec|") || key.starts_with("processes|") {
            true => Program::Running,
            false => Program::Launches,
        }
    }
}

impl Choice for Program {
    const COUNT: usize = Program::ALL.len();

    fn index(self) -> usize {
        Program::ALL
            .iter()
            .position(|program| *program == self)
            .unwrap_or(0)
    }

    fn step(self, by: isize, shown: &[Program]) -> Program {
        step_along(self, by, shown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_list_answers_for_its_own_collector_and_never_for_a_neighbour() {
        assert_eq!(Program::Running.collector(), "processes");
        assert_eq!(Program::Launches.collector(), "launches");
        assert_ne!(Program::Running.collector(), Program::Launches.collector());
    }

    #[test]
    fn the_object_of_a_finding_is_looked_for_in_the_list_that_holds_that_kind_of_key() {
        assert_eq!(
            Program::holding("exec|/usr/sbin/nginx|root"),
            Program::Running
        );
        assert_eq!(Program::holding("processes|unresolved"), Program::Running);
        assert_eq!(Program::holding("run|alice|/usr/bin/nc"), Program::Launches);
        assert_eq!(Program::holding("launches|dropping"), Program::Launches);
    }

    #[test]
    fn each_one_says_what_it_is_and_what_a_row_of_it_is_called() {
        for program in Program::ALL {
            assert!(!program.about().is_empty(), "{}", program.name());
            assert!(!program.caption().is_empty(), "{}", program.name());
            assert!(!program.detail().is_empty(), "{}", program.name());
        }
        assert!(
            Program::Launches.about().contains("never removed"),
            "a list that only grows has to say so: a reader takes a table for the host now"
        );
    }
}
