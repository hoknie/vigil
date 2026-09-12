#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sorting {
    pub by: usize,
    pub descending: bool,
}

pub const AS_READ: &str = "as the agent sends it";

impl Sorting {
    pub fn as_read(self) -> bool {
        self.by == 0
    }

    pub fn of(chosen: usize) -> Sorting {
        match chosen {
            0 => Sorting::default(),
            other => Sorting {
                by: (other - 1) / 2 + 1,
                descending: (other - 1) % 2 == 1,
            },
        }
    }

    pub fn chosen(self) -> usize {
        match self.by {
            0 => 0,
            by => (by - 1) * 2 + 1 + usize::from(self.descending),
        }
    }

    pub fn offered(columns: &[&str]) -> Vec<String> {
        let mut said = vec![AS_READ.to_string()];
        for column in columns.iter().skip(1) {
            said.push(format!("{column} ↑"));
            said.push(format!("{column} ↓"));
        }
        said
    }

    pub fn describe(self, columns: &[&str]) -> String {
        match columns.get(self.by) {
            Some(column) if !self.as_read() => format!(
                "{column}, {}",
                match self.descending {
                    true => "largest first",
                    false => "smallest first",
                }
            ),
            _ => AS_READ.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLUMNS: &[&str] = &[AS_READ, "TIME", "SEVERITY"];

    #[test]
    fn a_console_nobody_has_sorted_shows_the_list_in_the_order_the_agent_sent_it() {
        let fresh = Sorting::default();

        assert!(fresh.as_read());
        assert_eq!(fresh.chosen(), 0);
        assert_eq!(fresh.describe(COLUMNS), AS_READ);
    }

    #[test]
    fn every_column_is_offered_both_ways_round_and_the_choice_comes_back_the_same() {
        let offered = Sorting::offered(COLUMNS);

        assert_eq!(
            offered,
            vec![
                AS_READ.to_string(),
                "TIME ↑".to_string(),
                "TIME ↓".to_string(),
                "SEVERITY ↑".to_string(),
                "SEVERITY ↓".to_string(),
            ]
        );
        for chosen in 0..offered.len() {
            assert_eq!(
                Sorting::of(chosen).chosen(),
                chosen,
                "{chosen} does not survive being read back"
            );
        }
    }

    #[test]
    fn the_two_halves_of_a_column_are_the_same_column_facing_two_ways() {
        let up = Sorting::of(1);
        let down = Sorting::of(2);

        assert_eq!(up.by, down.by);
        assert!(!up.descending);
        assert!(down.descending);
        assert!(
            down.describe(COLUMNS).contains("TIME"),
            "{}",
            down.describe(COLUMNS)
        );
        assert!(down.describe(COLUMNS).contains("largest first"));
    }
}
