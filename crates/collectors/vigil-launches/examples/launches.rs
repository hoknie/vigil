use std::time::Instant;

use vigil_model::Snapshot;
#[cfg(target_os = "linux")]
use vigil_module::{Module, Settings};
use vigil_view::{Pane, Room, Section, Showing, Sorting, listed};

#[cfg(any(target_os = "linux", target_os = "macos"))]
const TAKEN_AT: &str = "2026-09-13T12:00:00.000Z";
const ROUNDS: u32 = 200;
const WINDOW: usize = 40;
const COPIES: [usize; 2] = [1, 1_000];
const SHORTER: &str = "n";
const LONGER: &str = "nc";

fn main() {
    reading();
    drawing();
    searching();
}

#[cfg(target_os = "linux")]
fn reading() {
    let settings = Settings::plain(|| TAKEN_AT.to_string());
    let collector = match vigil_launches::Launches.collector(&settings) {
        Ok(collector) => collector,
        Err(why) => {
            println!("launches: {why}");
            return;
        }
    };

    let started = Instant::now();
    let Ok(first) = collector.collect() else {
        println!(
            "launches: nothing to read here: {:?}",
            collector.available()
        );
        return;
    };
    println!(
        "launches: first reading {:.2} ms, {} items, {} bytes as the baseline",
        started.elapsed().as_secs_f64() * 1000.0,
        first.items.len(),
        serde_json::to_string(&first).expect("serialises").len()
    );

    let started = Instant::now();
    for _ in 0..ROUNDS {
        let _ = collector.collect();
    }
    println!(
        "launches: {:.2} ms per reading over {ROUNDS}",
        started.elapsed().as_secs_f64() * 1000.0 / f64::from(ROUNDS)
    );
}

#[cfg(target_os = "macos")]
fn reading() {
    use std::fs;

    use vigil_collect::Collector;
    use vigil_launches::{
        ESLOGGER_SPOOL, ESLOGGER_STATUS, Launched, LaunchesCollector, RUNNING, SpoolWriter,
        SpoolerStatus, audit_records,
    };

    const SPOOLED: u64 = 20_000;

    let printed = include_str!("../src/parsers/eslogger/tests/exec.ndjson").repeat(5_000);
    let started = Instant::now();
    let mut turned = 0usize;
    for line in printed.lines() {
        if let Ok(launched) = vigil_launches::parse_eslogger_event(line.as_bytes()) {
            turned += audit_records(&launched, true).len();
        }
    }
    println!(
        "launches: {} exec events eslogger printed ({} bytes) read and turned into {turned} bytes of records in {:.2} ms",
        printed.lines().count(),
        printed.len(),
        started.elapsed().as_secs_f64() * 1000.0
    );

    let at = std::env::temp_dir().join(format!("vigil-launches-cost-{}", std::process::id()));
    let _ = fs::remove_dir_all(&at);
    let _ = fs::create_dir_all(&at);
    let _ = SpoolerStatus {
        state: RUNNING.to_string(),
        since: TAKEN_AT.to_string(),
        program: "/usr/bin/eslogger".to_string(),
        arguments_recorded: true,
        why: None,
    }
    .write(&at.join(ESLOGGER_STATUS));

    let started = Instant::now();
    let mut writer = SpoolWriter::open(at.join(ESLOGGER_SPOOL), 64 * 1024 * 1024).expect("a spool");
    let mut pending: Vec<u8> = Vec::new();
    for sequence in 0..SPOOLED {
        let launched = Launched {
            seconds: 1_789_808_523 + sequence,
            milliseconds: 0,
            sequence,
            pid: 4000,
            ppid: 1,
            auid: 501,
            uid: 501,
            euid: 501,
            executable: format!("/usr/local/bin/tool{}", sequence % 300),
            arguments: vec![
                "tool".to_string(),
                "--password".to_string(),
                "hunter2".to_string(),
            ],
            working_directory: None,
        };
        pending.extend_from_slice(audit_records(&launched, true).as_bytes());
        if pending.len() >= 64 * 1024 {
            let _ = writer.write(&pending);
            pending.clear();
        }
    }
    let _ = writer.write(&pending);
    println!(
        "launches: {SPOOLED} launches spooled in {:.2} ms, {} bytes, written 64 KiB at a time",
        started.elapsed().as_secs_f64() * 1000.0,
        writer.bytes()
    );

    let collector = LaunchesCollector::with_paths(|| TAKEN_AT.to_string(), true, &at);
    let started = Instant::now();
    let mut rounds = 0;
    while collector
        .collect()
        .map(|read| read.items.len())
        .unwrap_or(0)
        > 0
        && rounds < 32
    {
        rounds += 1;
        let cursor =
            fs::read_to_string(at.join(format!("{ESLOGGER_SPOOL}.cursor"))).unwrap_or_default();
        if cursor
            .split_whitespace()
            .nth(1)
            .and_then(|offset| offset.parse::<u64>().ok())
            == Some(writer.bytes())
        {
            break;
        }
    }
    println!(
        "launches: the whole spool read in {} reading(s), {:.2} ms",
        rounds + 1,
        started.elapsed().as_secs_f64() * 1000.0
    );

    let started = Instant::now();
    for _ in 0..ROUNDS {
        let _ = collector.collect();
    }
    println!(
        "launches: {:.2} ms per reading with nothing new over {ROUNDS}",
        started.elapsed().as_secs_f64() * 1000.0 / f64::from(ROUNDS)
    );

    let started = Instant::now();
    let health = collector.available();
    println!(
        "launches: health {:.2} ms: {health:?}",
        started.elapsed().as_secs_f64() * 1000.0
    );
    let _ = fs::remove_dir_all(&at);
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn reading() {
    println!("launches: this reading is taken on Linux and on macOS; nothing to measure here");
}

fn drawing() {
    let reading = vigil_launches::fixture::launches();
    let room = Room::of(160);

    println!("pane          rows ms  cells ms (a window of {WINDOW})");
    for pane in vigil_launches::WhatHasRunHere.panes() {
        if !pane.shown(&reading) {
            continue;
        }
        let started = Instant::now();
        let mut rows = Vec::new();
        for _ in 0..ROUNDS {
            rows = pane.rows(&reading, &Showing::default());
        }
        let listing = started.elapsed().as_secs_f64() * 1_000.0 / f64::from(ROUNDS);

        let started = Instant::now();
        for _ in 0..ROUNDS {
            for row in rows.iter().take(WINDOW) {
                let _ = pane.cells(&reading, row, room);
            }
        }
        let cells = started.elapsed().as_secs_f64() * 1_000.0 / f64::from(ROUNDS);

        println!("{:<12}  {listing:>7.4}  {cells:>8.4}", pane.name());
    }
}

fn searching() {
    let read = vigil_launches::fixture::launches();
    for copies in COPIES {
        let reading = scaled(&read, copies);
        let rounds = (ROUNDS / u32::try_from(copies).unwrap_or(u32::MAX)).max(10);
        println!(
            "{} items, ms per call over {rounds} rounds; a keystroke types {LONGER:?} after {SHORTER:?}",
            reading.items.len()
        );
        println!(
            "pane          index  counts  key rows  key listed  sort rows  sort first  sort listed  tally  tally listed"
        );
        for pane in vigil_launches::WhatHasRunHere.panes() {
            if pane.shown(&reading) {
                measured(pane.as_ref(), &reading, rounds);
            }
        }
    }
}

fn measured(pane: &dyn Pane, reading: &Snapshot, rounds: u32) {
    let every = Showing::default();
    let (index_ms, index) = timed(rounds, || pane.index(reading, &every));
    let (counts_ms, counts) = timed(rounds, || pane.counts(reading, &every));
    let (Some(index), Some(counts)) = (index, counts) else {
        println!("{:<12}  builds no index or no counts", pane.name());
        return;
    };

    let shorter = Showing::searching(SHORTER);
    let longer = Showing::searching(LONGER);
    let (found, _) = listed(pane, reading, &shorter, &index, None);
    let (key_rows, _) = timed(rounds, || pane.rows(reading, &longer));
    let (key_listed, _) = timed(rounds, || {
        listed(pane, reading, &longer, &index, Some(&found))
    });

    let sorted = Showing::default().sorted(Sorting {
        by: 2,
        descending: true,
    });
    let (sort_rows, _) = timed(rounds, || pane.rows(reading, &sorted));
    let mut first = 0.0;
    for _ in 0..rounds {
        let Some(fresh) = pane.index(reading, &every) else {
            return;
        };
        first += timed(1, || listed(pane, reading, &sorted, &fresh, None)).0;
    }
    let first = first / f64::from(rounds);
    let (sort_listed, _) = timed(rounds, || listed(pane, reading, &sorted, &index, None));

    let rows = listed(pane, reading, &longer, &index, None).1;
    let (tally, _) = timed(rounds, || pane.tally(reading, &longer, rows.len()));
    let (tally_listed, _) = timed(rounds, || {
        pane.tally_listed(reading, &longer, &vigil_view::Rows::built(&rows), &counts)
    });

    let sorting = match pane.offers().sorting {
        true => format!("{sort_rows:>9.4}  {first:>10.4}  {sort_listed:>11.4}"),
        false => format!("{:>9}  {:>10}  {:>11}", "-", "-", "-"),
    };
    println!(
        "{:<12}  {index_ms:>5.3}  {counts_ms:>6.3}  {key_rows:>8.4}  {key_listed:>10.4}  {sorting}  {tally:>5.3}  {tally_listed:>12.4}",
        pane.name()
    );
}

fn timed<T>(rounds: u32, mut work: impl FnMut() -> T) -> (f64, T) {
    let started = Instant::now();
    let mut last = work();
    for _ in 1..rounds {
        last = work();
    }
    (
        started.elapsed().as_secs_f64() * 1_000.0 / f64::from(rounds),
        last,
    )
}

fn scaled(reading: &Snapshot, copies: usize) -> Snapshot {
    let mut scaled = reading.clone();
    for copy in 1..copies {
        for (key, item) in &reading.items {
            scaled.items.insert(format!("{key}~{copy}"), item.clone());
        }
    }
    scaled
}
