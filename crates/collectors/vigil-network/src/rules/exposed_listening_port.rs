use std::collections::BTreeMap;

use serde_json::Value;
use vigil_model::{Change, Evidence, KnownKind, Severity};

use super::socket_finding::{SocketFinding, build, owner_evidence};
use crate::types::SocketView;
use vigil_rules::{Batch, BatchRule, RuleContext};

pub struct ExposedListeningPort;

struct Side<'a> {
    key: &'a str,
    value: &'a Value,
    protocol: String,
    address: String,
}

impl BatchRule for ExposedListeningPort {
    fn name(&self) -> &'static str {
        "exposed_listening_port"
    }

    fn apply(&self, changes: &[Change], ctx: &mut RuleContext<'_>) -> Batch {
        let mut gone: BTreeMap<(&str, u64), Vec<Side<'_>>> = BTreeMap::new();
        let mut arrived: BTreeMap<(&str, u64), Vec<Side<'_>>> = BTreeMap::new();

        for change in changes {
            match change {
                Change::Removed { key, before } => {
                    let view = SocketView::new(before);
                    if view.loopback_only() {
                        gone.entry((view.transport(), view.port()))
                            .or_default()
                            .push(side(key, before, &view));
                    }
                }
                Change::Added { key, after } => {
                    let view = SocketView::new(after);
                    if view.world_reachable() {
                        arrived
                            .entry((view.transport(), view.port()))
                            .or_default()
                            .push(side(key, after, &view));
                    }
                }
                Change::Changed { .. } => {}
            }
        }

        let mut batch = Batch::silent();
        for ((transport, port), mut left) in gone {
            let Some(mut joined) = arrived.remove(&(transport, port)) else {
                continue;
            };

            left.sort_by(|a, b| (&a.protocol, &a.address).cmp(&(&b.protocol, &b.address)));
            joined.sort_by(|a, b| (&a.protocol, &a.address).cmp(&(&b.protocol, &b.address)));

            let was = &left[0];
            let now = joined
                .iter()
                .find(|side| side.protocol == was.protocol)
                .unwrap_or(&joined[0]);

            batch
                .findings
                .push(exposure(transport, port, was, now, &joined, ctx));
            batch
                .claimed
                .extend(left.iter().chain(joined.iter()).map(|s| s.key.to_string()));
        }

        batch
    }
}

fn side<'a>(key: &'a str, value: &'a Value, view: &SocketView<'a>) -> Side<'a> {
    Side {
        key,
        value,
        protocol: view.protocol().to_string(),
        address: view.address().to_string(),
    }
}

fn exposure(
    transport: &str,
    port: u64,
    was: &Side<'_>,
    now: &Side<'_>,
    joined: &[Side<'_>],
    ctx: &mut RuleContext<'_>,
) -> vigil_model::Finding {
    let (before, after) = (SocketView::new(was.value), SocketView::new(now.value));

    let mut evidence = owner_evidence(&after);
    if joined.len() > 1 {
        let addresses: Vec<&str> = joined.iter().map(|side| side.address.as_str()).collect();
        evidence.push(Evidence {
            kind: "note".into(),
            value: format!("now bound on {}", addresses.join(", ")),
        });
    }
    if before.owner_resolved()
        && after.owner_resolved()
        && (before.executable(), before.user()) != (after.executable(), after.user())
    {
        evidence.push(Evidence {
            kind: "note".into(),
            value: format!("was held by {}", before.describe_owner()),
        });
    }

    build(
        SocketFinding {
            kind: KnownKind::PortListenExposed,
            severity: match after.suspicious_executable() {
                true => Severity::Critical,
                false => Severity::High,
            },
            rule: "exposed_listening_port",
            key: now.key,
            title: format!(
                "{port}/{transport} moved from {} to {}, now reachable from the network ({})",
                was.address,
                now.address,
                after.describe_owner()
            ),
            before: Some(was.value.clone()),
            after: Some(now.value.clone()),
            evidence,
        },
        ctx,
    )
}
