use std::time::Instant;

use serde_json::json;
use vigil_model::{Change, Finding, Snapshot};
use vigil_module::{Module, Settings};
use vigil_rules::{RuleContext, RuleSet, diff};

const SIZES: &[usize] = &[100, 1_000, 10_000];
const CHANGES: &[usize] = &[0, 1, 10];
const ROUNDS: usize = 20;
const TAKEN_AT: &str = "2026-09-10T12:00:00.000Z";

fn main() {
    reading();
    judging();
    drawing();
}

fn drawing() {
    use vigil_view::{Room, Section, Showing};

    const WINDOW: usize = 40;
    let room = Room::of(160);

    println!("items  rows ms  cells ms (a window of {WINDOW})");
    for size in SIZES {
        let reading = host(*size, 0);
        for pane in vigil_network::Listening.panes() {
            let started = Instant::now();
            let mut rows = Vec::new();
            for _ in 0..ROUNDS {
                rows = pane.rows(&reading, &Showing::default());
            }
            let listing = started.elapsed().as_secs_f64() * 1_000.0 / ROUNDS as f64;

            let started = Instant::now();
            for _ in 0..ROUNDS {
                for row in rows.iter().take(WINDOW) {
                    let _ = pane.cells(&reading, row, room);
                }
            }
            let cells = started.elapsed().as_secs_f64() * 1_000.0 / ROUNDS as f64;

            println!("{size:>5}  {listing:>7.3}  {cells:>8.3}  {}", pane.name());
        }
    }
}

#[cfg(target_os = "linux")]
fn reading() {
    use vigil_collect::Collector;
    use vigil_network::PortsCollector;

    let rounds: u32 = 500;
    let collector = PortsCollector::new(|| TAKEN_AT.to_string());

    let started = Instant::now();
    let Ok(first) = collector.collect() else {
        println!(
            "ports: nothing to read on this host: {:?}",
            collector.available()
        );
        return;
    };
    println!(
        "ports: first reading {:.2} ms, {} items, {} bytes as the baseline",
        started.elapsed().as_secs_f64() * 1000.0,
        first.items.len(),
        serde_json::to_string(&first).expect("serialises").len()
    );

    let started = Instant::now();
    for _ in 0..rounds {
        let _ = collector.collect();
    }
    println!(
        "ports: {:.2} ms per reading over {rounds}",
        started.elapsed().as_secs_f64() * 1000.0 / f64::from(rounds)
    );
}

#[cfg(not(target_os = "linux"))]
fn reading() {
    println!("ports: the reading walks a Linux /proc; nothing to measure here");
}

fn judging() {
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
            let rules = vigil_network::Ports.rules(&Settings::plain(|| TAKEN_AT.to_string()));

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
