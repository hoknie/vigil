use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts {
    numbers: BTreeMap<&'static str, usize>,
    words: BTreeMap<&'static str, Vec<String>>,
}

impl Counts {
    pub fn counted(mut self, name: &'static str, number: usize) -> Counts {
        self.numbers.insert(name, number);
        self
    }

    pub fn saying(mut self, name: &'static str, words: Vec<String>) -> Counts {
        self.words.insert(name, words);
        self
    }

    pub fn number(&self, name: &str) -> usize {
        self.numbers.get(name).copied().unwrap_or_default()
    }

    pub fn words(&self, name: &str) -> &[String] {
        self.words.get(name).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_nobody_took_is_nothing_and_not_a_panic() {
        let counts = Counts::default()
            .counted("accounts", 3)
            .saying("sources", vec!["logind: read, 2".to_string()]);

        assert_eq!(counts.number("accounts"), 3);
        assert_eq!(counts.number("groups"), 0);
        assert_eq!(counts.words("sources"), ["logind: read, 2"]);
        assert!(counts.words("sockets").is_empty());
    }
}
