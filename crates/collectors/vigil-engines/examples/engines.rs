use std::time::Instant;

use vigil_engines::fixture;
use vigil_model::Snapshot;
use vigil_view::{Room, Section, Showing, listed};

const ROUNDS: u32 = 200;

const WINDOW: usize = 40;

const COPIES: [usize; 4] = [1, 10, 50, 200];

fn main() {
    projecting();
    judging();
    listing();
    reading();
}

fn projecting() {
    let docker = fixture::docker::dump();
    let podman = fixture::podman::dump();

    println!("engines: two dumps in, one snapshot out");
    println!("copies   rows  bytes in  ms per round");
    for copies in COPIES {
        let docker = grown(&docker, copies);
        let podman = grown(&podman, copies);
        let bytes: usize = docker
            .asked
            .values()
            .chain(podman.asked.values())
            .map(|answer| answer.printed.len())
            .sum();

        let started = Instant::now();
        let mut rows = 0;
        for _ in 0..ROUNDS {
            rows = fixture::read(&docker, &podman).items.len();
        }
        let each = started.elapsed().as_secs_f64() * 1_000.0 / f64::from(ROUNDS);

        println!("{copies:>6}  {rows:>5}  {bytes:>8}  {each:>12.4}");
    }
}

fn judging() {
    let docker = fixture::docker::dump();
    let podman = fixture::podman::dump();
    let module = vigil_engines::Engines;
    let settings = vigil_module::Settings::plain(|| "2026-09-18T09:00:00.000Z".to_string());

    println!("engines: the rules over one round, the diff included");
    println!("copies   rows  quiet ms  moved ms  findings  arrived ms  findings");
    for copies in COPIES {
        let before = fixture::read(&grown(&docker, copies), &grown(&podman, copies));
        let after = moved(&before);
        let bare = engines_alone(&before);
        let rules = vigil_module::Module::rules(&module, &settings);

        let (_, quiet) = timed(|| judged(&rules, &before, &before));
        let (moved, busy) = timed(|| judged(&rules, &before, &after));
        let (arrived, all_at_once) = timed(|| judged(&rules, &bare, &before));

        println!(
            "{copies:>6}  {:>5}  {quiet:>8.4}  {busy:>8.4}  {moved:>8}  {all_at_once:>10.4}  \
             {arrived:>8}",
            before.items.len()
        );
    }
}

fn engines_alone(reading: &Snapshot) -> Snapshot {
    let mut bare = reading.clone();
    bare.items.retain(|key, _| key.contains("|engine|"));
    bare
}

fn moved(reading: &Snapshot) -> Snapshot {
    let mut after = reading.clone();
    let keys: Vec<String> = after.items.keys().cloned().collect();
    for (at, key) in keys.iter().enumerate() {
        match at % 7 {
            0 => {
                after.items.remove(key);
            }
            1 => {
                if let Some(item) = after.items.get_mut(key) {
                    item["subnets"] = serde_json::json!(["10.99.0.0/24"]);
                    item["services"] = serde_json::json!(["added"]);
                    item["driver"] = serde_json::json!("moved");
                }
            }
            _ => {}
        }
    }
    after
}

fn judged(rules: &vigil_rules::RuleSet, before: &Snapshot, after: &Snapshot) -> usize {
    let mut mint = || "event".to_string();
    let mut ctx = vigil_rules::RuleContext {
        now: "2026-09-18T09:00:00.000Z".into(),
        mint_event_id: &mut mint,
    };
    rules
        .judge(&vigil_rules::diff(before, after), &mut ctx)
        .len()
}

fn listing() {
    let docker = grown(&fixture::docker::dump(), 200);
    let podman = grown(&fixture::podman::dump(), 200);
    let reading = fixture::read(&docker, &podman);
    let keystroke = Showing::searching("shop");
    let room = Room::of(120);

    println!("engines: every list at {} rows", reading.items.len());
    println!(
        "list                  rows  index ms  counts ms  rows ms  listed ms  cells ms  detail ms"
    );
    for pane in vigil_engines::WhatTheEnginesHold.panes() {
        let pane = pane.as_ref();
        let (index, indexing) = timed(|| {
            pane.index(&reading, &Showing::default())
                .expect("every list answers from an index")
        });
        let (_, counting) = timed(|| pane.counts(&reading, &Showing::default()));
        let (rows, by_rows) = timed(|| pane.rows(&reading, &keystroke));
        let (_, by_index) = timed(|| listed(pane, &reading, &keystroke, &index, None));
        let every = pane.rows(&reading, &Showing::default());
        let (_, cells) = timed(|| {
            for row in every.iter().take(WINDOW) {
                let _ = pane.cells(&reading, row, room);
            }
        });
        let (_, detail) = timed(|| {
            every
                .first()
                .map(|row| pane.detail(&reading, row, 120).len())
                .unwrap_or_default()
        });

        println!(
            "{:<8}{:<12}  {:>4}  {indexing:>8.4}  {counting:>9.4}  {by_rows:>7.4}  {by_index:>9.4}  \
             {cells:>8.4}  {detail:>9.4}",
            pane.belongs_to().unwrap_or_default(),
            pane.name(),
            every.len().max(rows.len())
        );
    }
}

fn timed<T>(mut work: impl FnMut() -> T) -> (T, f64) {
    let started = Instant::now();
    let mut last = work();
    for _ in 1..ROUNDS {
        last = work();
    }
    (
        last,
        started.elapsed().as_secs_f64() * 1_000.0 / f64::from(ROUNDS),
    )
}

fn grown(dump: &vigil_engines::Dump, copies: usize) -> vigil_engines::Dump {
    let mut grown = dump.clone();

    for answer in grown.asked.values_mut() {
        let one = answer.printed.clone();
        let mut printed = String::with_capacity(one.len() * copies);
        for copy in 0..copies {
            printed.push_str(
                &one.replace("sha256:", &format!("sha256:{copy:04}"))
                    .replace("-1\"", &format!("-1-{copy:04}\"")),
            );
        }
        answer.printed = printed;
    }

    grown
}

#[cfg(target_os = "linux")]
fn reading() {
    use std::fs;

    use vigil_collect::Collector;
    use vigil_engines::{EnginesCollector, Watching};

    let at = std::env::temp_dir().join(format!("vigil-engines-cost-{}", std::process::id()));
    let _ = fs::create_dir_all(&at);
    for (dump, named) in [
        (fixture::docker::dump(), "docker.json"),
        (fixture::podman::dump(), "podman.json"),
    ] {
        let _ = fs::write(
            at.join(named),
            serde_json::to_vec_pretty(&dump).expect("plain data"),
        );
    }

    let collector = EnginesCollector::with_paths(
        || "2026-09-17T09:00:00.000Z".to_string(),
        &at,
        Watching::default(),
    );

    let started = Instant::now();
    let first = match collector.collect() {
        Ok(first) => first,
        Err(why) => {
            println!("engines: nothing to read here: {why}");
            let _ = fs::remove_dir_all(&at);
            return;
        }
    };
    println!(
        "engines: first reading {:.2} ms, {} items, {} bytes as the baseline",
        started.elapsed().as_secs_f64() * 1000.0,
        first.items.len(),
        serde_json::to_string(&first).expect("serialises").len()
    );

    let started = Instant::now();
    for _ in 0..ROUNDS {
        let _ = collector.collect();
    }
    println!(
        "engines: {:.2} ms per reading over {ROUNDS}, reading the dump off disk each time",
        started.elapsed().as_secs_f64() * 1000.0 / f64::from(ROUNDS)
    );

    let _ = fs::remove_dir_all(&at);
}

#[cfg(not(target_os = "linux"))]
fn reading() {
    println!("engines: this reading is taken on Linux; nothing to measure here");
}
