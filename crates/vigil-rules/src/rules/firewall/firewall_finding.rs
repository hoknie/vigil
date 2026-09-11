use serde_json::{Value, json};
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use super::firewall_view::{Family, FirewallView};
use crate::RuleContext;

pub struct FirewallFinding<'a> {
    pub kind: KnownKind,
    pub severity: Severity,
    pub rule: &'static str,
    pub key: &'a str,
    pub object: &'static str,
    pub title: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub evidence: Vec<Evidence>,
}

const FINDING_KEY_PREFIX: &str = "firewall";

const SNAPSHOT_KEY_PREFIX: &str = "fw-";

pub fn build(spec: FirewallFinding<'_>, ctx: &mut RuleContext<'_>) -> Finding {
    let describing = spec.after.as_ref().or(spec.before.as_ref());
    let subject = match describing {
        Some(value) => {
            let view = FirewallView::new(spec.key, value);
            match view.family() {
                Some(Family::Table) => json!({
                    "family": view.network_family(),
                    "table": view.name(),
                }),
                Some(Family::Chain) => json!({
                    "family": view.network_family(),
                    "table": view.table(),
                    "chain": view.name(),
                    "hook": view.hook(),
                }),
                _ => json!({ "backend": "nftables" }),
            }
        }
        None => json!({ "key": spec.key }),
    };

    let now = ctx.now.clone();
    Finding {
        event_id: (ctx.mint_event_id)(),
        finding_key: finding_key(spec.key),
        kind: Kind::Known(spec.kind),
        severity: spec.severity,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: spec.title,
        subject: Subject {
            object: spec.object.into(),
            key: subject,
        },
        before: spec.before,
        after: spec.after,
        evidence: spec.evidence,
        redacted: Vec::new(),
        rule: Some(spec.rule.to_string()),
        labels: Default::default(),
    }
}

pub fn finding_key(key: &str) -> String {
    format!(
        "{FINDING_KEY_PREFIX}|{}",
        key.strip_prefix(SNAPSHOT_KEY_PREFIX).unwrap_or(key)
    )
}

pub fn counted(view: &FirewallView<'_>) -> Evidence {
    Evidence {
        kind: "note".into(),
        value: format!(
            "{} table(s), {} chain(s) in all, {} rule(s), {} chain(s) on the input hook; families {}",
            view.tables(),
            view.chains(),
            view.rules(),
            view.hooked_on_input(),
            match view.families().is_empty() {
                true => "none".to_string(),
                false => view.families().join(", "),
            }
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finding_key_names_the_product_word_and_not_the_short_one_the_snapshot_uses() {
        assert_eq!(
            finding_key("fw-summary|nftables"),
            "firewall|summary|nftables"
        );
        assert_eq!(
            finding_key("fw-table|inet filter"),
            "firewall|table|inet filter"
        );
        assert_eq!(
            finding_key("fw-chain|inet filter|input"),
            "firewall|chain|inet filter|input",
            "a person writing a suppression reads the family name the contract uses, and one \
             prefix for the whole family is what a prefix suppression bites on"
        );
        assert_eq!(
            finding_key("fw-backend|legacy"),
            "firewall|backend|legacy",
            "the short word the snapshot key opens with is the only part replaced; the class \
             that follows it stays, so firewall|chain| bites chains and firewall| bites all"
        );
    }
}
