#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Reads,
    Concludes,
}

impl Group {
    pub const ALL: &'static [Group] = &[Group::Reads, Group::Concludes];

    pub fn caption(self) -> &'static str {
        match self {
            Group::Reads => "WHAT IT READS",
            Group::Concludes => "WHAT IT CONCLUDES",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_the_agent_reads_is_named_before_what_it_makes_of_it() {
        assert_eq!(Group::ALL, &[Group::Reads, Group::Concludes]);
        for group in Group::ALL {
            assert!(!group.caption().is_empty());
        }
    }
}
