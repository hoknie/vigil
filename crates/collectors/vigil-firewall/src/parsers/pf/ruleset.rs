use super::rule::{Direction, PfRule};

pub const MAIN: &str = "main";

const DROP: &str = "drop";

const ACCEPT: &str = "accept";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PfRuleset {
    pub name: String,
    pub filter: Vec<PfRule>,
    pub translation: Vec<PfRule>,
}

impl PfRuleset {
    pub fn is_the_main_one(&self) -> bool {
        self.name == MAIN
    }

    pub fn rules(&self) -> usize {
        self.filter.len() + self.translation.len()
    }

    pub fn filtering(&self, direction: Direction) -> Vec<&PfRule> {
        self.filter
            .iter()
            .filter(|rule| rule.applies_to(direction))
            .collect()
    }

    pub fn policy(&self, direction: Direction) -> &'static str {
        let mut decided = ACCEPT;
        for rule in self.filtering(direction) {
            if !rule.filters() || !rule.unconditional {
                continue;
            }
            decided = match rule.action.as_str() {
                "block" => DROP,
                _ => ACCEPT,
            };
            if rule.quick {
                break;
            }
        }
        decided
    }

    pub fn redirecting(&self) -> Vec<&PfRule> {
        self.translation
            .iter()
            .filter(|rule| rule.redirects())
            .collect()
    }

    pub fn translating_out(&self) -> Vec<&PfRule> {
        self.translation
            .iter()
            .filter(|rule| !rule.redirects())
            .collect()
    }
}
