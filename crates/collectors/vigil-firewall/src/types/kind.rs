#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Ruleset,
    Backend,
    Table,
    Chain,
}

const RULESET: &str = "fw-summary";

const TABLE: &str = "fw-table";

const CHAIN: &str = "fw-chain";

const BACKEND: &str = "fw-backend";

impl Kind {
    pub fn of(key: &str) -> Option<Kind> {
        match key.split('|').next()? {
            RULESET => Some(Kind::Ruleset),
            TABLE => Some(Kind::Table),
            CHAIN => Some(Kind::Chain),
            BACKEND => Some(Kind::Backend),
            _ => None,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Kind::Ruleset => RULESET,
            Kind::Backend => BACKEND,
            Kind::Table => TABLE,
            Kind::Chain => CHAIN,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Ruleset => "ruleset",
            Kind::Backend => "backend",
            Kind::Table => "table",
            Kind::Chain => "chain",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_row_of_the_reading_is_named_by_the_word_its_key_opens_with() {
        assert_eq!(Kind::of("fw-summary|nftables"), Some(Kind::Ruleset));
        assert_eq!(Kind::of("fw-table|inet filter"), Some(Kind::Table));
        assert_eq!(Kind::of("fw-chain|inet filter|input"), Some(Kind::Chain));
        assert_eq!(Kind::of("fw-backend|legacy"), Some(Kind::Backend));
    }

    #[test]
    fn a_row_of_another_collector_is_nothing_this_screen_draws() {
        assert_eq!(Kind::of("tcp|0.0.0.0:443"), None);
        assert_eq!(Kind::of("unit|nginx.service"), None);
        assert_eq!(
            Kind::of("fw-rule|inet filter|input|4"),
            None,
            "there is no row per rule in this reading, and a screen that draws one is drawing \
             a key the agent never sends"
        );
    }

    #[test]
    fn the_whole_ruleset_is_read_before_the_tables_it_is_made_of() {
        let mut order = vec![Kind::Chain, Kind::Table, Kind::Ruleset, Kind::Backend];
        order.sort();

        assert_eq!(
            order,
            vec![Kind::Ruleset, Kind::Backend, Kind::Table, Kind::Chain],
            "a reader looks at what the host filters as a whole, then at what holds the rules"
        );
    }
}
