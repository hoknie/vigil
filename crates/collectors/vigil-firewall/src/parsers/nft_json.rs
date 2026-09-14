use std::fmt;

use serde_json::Value;

const FIRST_LINE_SHOWN: usize = 160;

const ROOT: &str = "nftables";

const METAINFO: &str = "metainfo";

const TABLE: &str = "table";

const CHAIN: &str = "chain";

const RULE: &str = "rule";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NftRefusal {
    Empty,
    NotJson(String),
    NotARuleset(String),
}

impl fmt::Display for NftRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NftRefusal::Empty => write!(
                f,
                "the ruleset file is empty: nft was started and wrote nothing, so what this host filters is unknown"
            ),
            NftRefusal::NotJson(head) => write!(
                f,
                "the ruleset file is not JSON, it begins {head:?}: what this host filters is unknown"
            ),
            NftRefusal::NotARuleset(why) => write!(
                f,
                "the ruleset file is JSON in a shape this build does not know ({why}): what this host filters is unknown"
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NftTable {
    pub family: String,
    pub name: String,
    pub chains: usize,
    pub rules: usize,
}

impl NftTable {
    pub fn key(&self) -> String {
        format!("{} {}", self.family, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NftChain {
    pub family: String,
    pub table: String,
    pub name: String,
    pub kind: String,
    pub hook: String,
    pub priority: i64,
    pub policy: String,
    pub rules: usize,
}

impl NftChain {
    pub fn table_key(&self) -> String {
        format!("{} {}", self.family, self.table)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NftRuleset {
    pub version: Option<String>,
    pub tables: Vec<NftTable>,
    pub base_chains: Vec<NftChain>,
    pub rules: usize,
}

impl NftRuleset {
    pub fn families(&self) -> Vec<String> {
        let mut families: Vec<String> = self
            .tables
            .iter()
            .map(|table| table.family.clone())
            .collect();
        families.sort_unstable();
        families.dedup();
        families
    }

    pub fn chains(&self) -> usize {
        self.tables.iter().map(|table| table.chains).sum()
    }

    pub fn hooked_on_input(&self) -> usize {
        self.base_chains
            .iter()
            .filter(|chain| chain.hook == "input")
            .count()
    }
}

pub fn parse_nft_ruleset(bytes: &[u8]) -> Result<NftRuleset, NftRefusal> {
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Err(NftRefusal::Empty);
    }

    let document: Value =
        serde_json::from_slice(bytes).map_err(|_| NftRefusal::NotJson(first_line(bytes)))?;

    let Some(elements) = document.get(ROOT).and_then(Value::as_array) else {
        return Err(NftRefusal::NotARuleset(format!(
            "no {ROOT:?} list at the top"
        )));
    };

    let mut ruleset = NftRuleset::default();
    let mut chains: Vec<NftChain> = Vec::new();
    let mut described = 0usize;

    for element in elements {
        let Some((name, body)) = element.as_object().and_then(|fields| fields.iter().next()) else {
            continue;
        };
        match name.as_str() {
            METAINFO => {
                ruleset.version = body
                    .get("version")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                described += 1;
            }
            TABLE => ruleset.tables.push(NftTable {
                family: text(body, "family"),
                name: text(body, "name"),
                chains: 0,
                rules: 0,
            }),
            CHAIN => chains.push(NftChain {
                family: text(body, "family"),
                table: text(body, "table"),
                name: text(body, "name"),
                kind: text(body, "type"),
                hook: text(body, "hook"),
                priority: body.get("prio").and_then(Value::as_i64).unwrap_or(0),
                policy: text(body, "policy"),
                rules: 0,
            }),
            RULE => {
                ruleset.rules += 1;
                let table = format!("{} {}", text(body, "family"), text(body, "table"));
                let chain = text(body, "chain");
                for held in chains.iter_mut() {
                    if held.table_key() == table && held.name == chain {
                        held.rules += 1;
                    }
                }
                for held in ruleset.tables.iter_mut() {
                    if held.key() == table {
                        held.rules += 1;
                    }
                }
            }
            _ => {}
        }
    }

    if described == 0 {
        return Err(NftRefusal::NotARuleset(format!(
            "no {METAINFO:?} element, which every nft --json ruleset carries"
        )));
    }

    for chain in &chains {
        let table = chain.table_key();
        for held in ruleset.tables.iter_mut() {
            if held.key() == table {
                held.chains += 1;
            }
        }
    }

    ruleset.base_chains = chains
        .into_iter()
        .filter(|chain| !chain.hook.is_empty())
        .collect();
    ruleset
        .tables
        .sort_by(|left, right| (&left.family, &left.name).cmp(&(&right.family, &right.name)));

    Ok(ruleset)
}

fn text(body: &Value, field: &str) -> String {
    body.get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn first_line(bytes: &[u8]) -> String {
    let line = bytes
        .split(|byte| *byte == b'\n')
        .find(|line| !line.iter().all(u8::is_ascii_whitespace))
        .unwrap_or_default();
    let shown = &line[..line.len().min(FIRST_LINE_SHOWN)];
    String::from_utf8_lossy(shown).trim().to_string()
}
