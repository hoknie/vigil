#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    In,
    Out,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PfRule {
    pub number: usize,
    pub action: String,
    pub direction: Direction,
    pub quick: bool,
    pub unconditional: bool,
    pub matches: String,
}

const MODIFIERS: &[&str] = &[
    "drop",
    "return",
    "return-rst",
    "return-icmp",
    "return-icmp6",
    "in",
    "out",
    "log",
    "quick",
];

const OPTIONS_OF_AN_UNCONDITIONAL_RULE: &[&str] = &[
    "flags", "S/SA", "S/FSRA", "any", "keep", "modulate", "synproxy", "no", "state",
];

const TRANSLATION: &[&str] = &[
    "nat",
    "rdr",
    "binat",
    "nat-anchor",
    "rdr-anchor",
    "binat-anchor",
];

impl PfRule {
    pub fn applies_to(&self, direction: Direction) -> bool {
        self.direction == Direction::Both || self.direction == direction
    }

    pub fn filters(&self) -> bool {
        self.action == "pass" || self.action == "block"
    }

    pub fn translates(&self) -> bool {
        TRANSLATION.contains(&self.action.as_str())
    }

    pub fn redirects(&self) -> bool {
        self.action.starts_with("rdr") || self.action.starts_with("binat")
    }

    pub fn does(&self) -> String {
        match self.quick {
            true => format!("{} quick", self.action),
            false => self.action.clone(),
        }
    }
}

pub fn parse_pf_rules(printed: &str) -> Vec<PfRule> {
    printed
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('[') && !line.starts_with("No ALTQ"))
        .filter(|line| !line.starts_with("ALTQ"))
        .enumerate()
        .filter_map(|(at, line)| rule_of(at + 1, line))
        .collect()
}

fn rule_of(number: usize, line: &str) -> Option<PfRule> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let (first, rest) = words.split_first()?;
    let (action, rest) = match *first {
        "no" => (format!("no {}", rest.first()?), rest.get(1..)?),
        word => (word.to_string(), rest),
    };

    let modifiers = rest
        .iter()
        .take_while(|word| MODIFIERS.contains(word) || word.starts_with('('))
        .count();
    let (said, matches) = rest.split_at(modifiers);

    let direction = match (said.contains(&"in"), said.contains(&"out")) {
        (true, false) => Direction::In,
        (false, true) => Direction::Out,
        _ => Direction::Both,
    };
    let unconditional = matches.first() == Some(&"all")
        && matches[1..]
            .iter()
            .all(|word| OPTIONS_OF_AN_UNCONDITIONAL_RULE.contains(word));

    Some(PfRule {
        number,
        action,
        direction,
        quick: said.contains(&"quick"),
        unconditional,
        matches: matches.join(" "),
    })
}
