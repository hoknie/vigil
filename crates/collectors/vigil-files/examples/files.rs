use std::time::Instant;

use vigil_model::Snapshot;
#[cfg(target_os = "linux")]
use vigil_module::{Module, Settings};
use vigil_view::{Pane, Room, Section, Showing, Sorting, listed};

#[cfg(target_os = "linux")]
const TAKEN_AT: &str = "2026-09-13T12:00:00.000Z";
const ROUNDS: u32 = 200;
const WINDOW: usize = 40;
const COPIES: [usize; 2] = [1, 1_000];
const SHORTER: &str = "u";
const LONGER: &str = "us";

fn main() {
    reading();
    drawing();
    searching();
}

#[cfg(target_os = "linux")]
fn reading() {
    let settings = Settings::plain(|| TAKEN_AT.to_string());
    let collector = match vigil_files::Files.collector(&settings) {
        Ok(collector) => collector,
        Err(why) => {
            println!("files: {why}");
            return;
        }
    };

    let started = Instant::now();
    let Ok(first) = collector.collect() else {
        println!("files: nothing to read here: {:?}", collector.available());
        return;
    };
    println!(
        "files: first reading {:.2} ms, {} items, {} bytes as the baseline",
        started.elapsed().as_secs_f64() * 1000.0,
        first.items.len(),
        serde_json::to_string(&first).expect("serialises").len()
    );

    let started = Instant::now();
    for _ in 0..ROUNDS {
        let _ = collector.collect();
    }
    println!(
        "files: {:.2} ms per reading over {ROUNDS}",
        started.elapsed().as_secs_f64() * 1000.0 / f64::from(ROUNDS)
    );
}

#[cfg(not(target_os = "linux"))]
fn reading() {
    println!("files: this reading is taken on Linux; nothing to measure here");
}

fn drawing() {
    let reading = vigil_files::fixture::files();
    let room = Room::of(160);

    println!("pane          rows ms  cells ms (a window of {WINDOW})");
    for pane in vigil_files::TheHostAndItsFiles.panes() {
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
    let read = vigil_files::fixture::files();
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
        for pane in vigil_files::TheHostAndItsFiles.panes() {
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
