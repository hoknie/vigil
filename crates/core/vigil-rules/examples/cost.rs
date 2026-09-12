use std::time::Instant;

use serde_json::json;
use vigil_model::{Change, Finding, Snapshot};
use vigil_rules::{RuleContext, RuleSet, diff, listening_port_rules};

const SIZES: &[usize] = &[100, 1_000, 10_000];
const CHANGES: &[usize] = &[0, 1, 10];
const ROUNDS: usize = 20;
const TAKEN_AT: &str = "2026-09-10T12:00:00.000Z";

fn main() {
    let rounds: usize = std::env::args()
        .nth(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(ROUNDS)
        .max(1);

    println!("items  moved  diff ms  judge ms  tick ms  changes  findings  snapshot bytes");
    for size in SIZES {
        let before = host(*size, 0);
        let bytes = serde_json::to_string(&before).expect("serialises").len();

        for moved in CHANGES {
            let after = host(*size, *moved);
            let rules = listening_port_rules();

            let started = Instant::now();
            let mut changes = Vec::new();
            for _ in 0..rounds {
                changes = diff(&before, &after);
            }
            let differing = started.elapsed().as_secs_f64() * 1_000.0 / rounds as f64;

            let started = Instant::now();
            let mut findings = 0;
            for _ in 0..rounds {
                findings = judge(&rules, &changes).len();
            }
            let judging = started.elapsed().as_secs_f64() * 1_000.0 / rounds as f64;

            println!(
                "{size:>5}  {moved:>5}  {differing:>7.3}  {judging:>8.3}  {:>7.3}  {:>7}  {findings:>8}  {bytes:>14}",
                differing + judging,
                changes.len()
            );
        }
    }
}

fn judge(rules: &RuleSet, changes: &[Change]) -> Vec<Finding> {
    let mut minted: u64 = 0;
    let mut mint = move || {
        minted += 1;
        format!("0199a1b2-c3d4-7e5f-8a9b-{minted:012x}")
    };
    let mut ctx = RuleContext {
        now: TAKEN_AT.to_string(),
        mint_event_id: &mut mint,
    };

    rules.judge(changes, &mut ctx)
}

fn host(items: usize, moved: usize) -> Snapshot {
    let mut snapshot = Snapshot::new("ports", TAKEN_AT);

    for number in moved..items + moved {
        let octet = number / 250;
        let port = 1_024 + (number % 250) as u64;
        snapshot.items.insert(
            format!("tcp|10.0.0.{octet}:{port}"),
            socket(port, number < 2 * moved),
        );
    }
    snapshot
}

fn socket(port: u64, from_a_writable_path: bool) -> serde_json::Value {
    let exe = match from_a_writable_path {
        true => "/tmp/.x/nc",
        false => "/usr/sbin/nginx",
    };

    json!({
        "protocol": "tcp",
        "address": "10.0.0.1",
        "port": port,
        "uid": 0,
        "user": "root",
        "process": {
            "exe": exe,
            "exe_deleted": false,
            "cmdline": "nginx -g daemon off;",
            "cmdline_redacted": false,
        },
        "owner_resolved": true,
    })
}
