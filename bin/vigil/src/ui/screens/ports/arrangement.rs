use crate::ui::{Choice, step_along};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Arrangement {
    #[default]
    Flat,
    ByProgram,
}

impl Arrangement {
    pub const ALL: &'static [Arrangement] = &[Arrangement::Flat, Arrangement::ByProgram];

    pub fn name(self) -> &'static str {
        match self {
            Arrangement::Flat => "sockets",
            Arrangement::ByProgram => "by program",
        }
    }

    pub fn caption(self) -> &'static str {
        match self {
            Arrangement::Flat => "LISTENING",
            Arrangement::ByProgram => "BY PROGRAM",
        }
    }

    pub fn about(self) -> &'static str {
        match self {
            Arrangement::Flat => {
                "one row per listening socket: the reading itself, and what a finding is keyed \
                 by"
            }
            Arrangement::ByProgram => {
                "one row per program, with its sockets under it; sockets with no resolved \
                 owner are kept apart"
            }
        }
    }
}

impl Choice for Arrangement {
    const COUNT: usize = Arrangement::ALL.len();

    fn index(self) -> usize {
        Arrangement::ALL
            .iter()
            .position(|arrangement| *arrangement == self)
            .unwrap_or(0)
    }

    fn step(self, by: isize, shown: &[Arrangement]) -> Arrangement {
        step_along(self, by, shown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_second_view_of_the_ports_is_not_called_after_a_section_of_its_own() {
        assert_eq!(
            Arrangement::ByProgram.name(),
            "by program",
            "`programs` is a section now, and one word naming two places reads as one place"
        );
    }
}
