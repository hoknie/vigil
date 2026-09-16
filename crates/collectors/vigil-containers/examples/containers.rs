use std::time::Instant;

use vigil_model::Snapshot;
#[cfg(target_os = "linux")]
use vigil_module::{Module, Settings};
use vigil_view::{Room, Section, Showing, Sorting, listed};

#[cfg(target_os = "linux")]
const TAKEN_AT: &str = "2026-09-13T12:00:00.000Z";
const ROUNDS: u32 = 200;
const COPIES: [usize; 3] = [1, 100, 1_000];
const WINDOW: usize = 40;

fn main() {
    reading();
    drawing();
    searching();
}

fn searching() {
    let sorted = Showing::default().sorted(Sorting {
        by: 2,
        descending: true,
    });
    let keystroke = Showing::searching("nginx");
    println!(
        "items  index ms  counts ms  key rows ms  key listed ms  sort rows ms  sort cold ms  \
         sort warm ms  tally ms  tally listed ms"
    );
    for copies in COPIES {
        let reading = copied(&vigil_containers::fixture::containers(), copies);
        for pane in vigil_containers::WhatRunsInContainers.panes() {
            let pane = pane.as_ref();
            let (index, indexing) = timed(|| {
                pane.index(&reading, &Showing::default())
                    .expect("the pane answers from an index")
            });
            let (counts, counting) = timed(|| {
                pane.counts(&reading, &Showing::default())
                    .expect("the pane counts its reading once")
            });
            let (rows, by_rows) = timed(|| pane.rows(&reading, &keystroke));
            let (_, by_index) = timed(|| listed(pane, &reading, &keystroke, &index, None));
            let (_, sort_rows) = timed(|| pane.rows(&reading, &sorted));
            let fresh = pane
                .index(&reading, &Showing::default())
                .expect("the pane answers from an index");
            let started = Instant::now();
            let _ = listed(pane, &reading, &sorted, &fresh, None);
            let sort_cold = started.elapsed().as_secs_f64() * 1_000.0;
            let (_, sort_warm) = timed(|| listed(pane, &reading, &sorted, &fresh, None));
            let (_, tally) = timed(|| pane.tally(&reading, &keystroke, rows.len()));
            let (_, tally_listed) = timed(|| {
                pane.tally_listed(
                    &reading,
                    &keystroke,
                    &vigil_view::Rows::built(&rows),
                    &counts,
                )
            });

            println!(
                "{:>5}  {indexing:>8.4}  {counting:>9.4}  {by_rows:>11.4}  {by_index:>13.4}  \
                 {sort_rows:>12.4}  {sort_cold:>12.4}  {sort_warm:>12.4}  {tally:>8.4}  \
                 {tally_listed:>15.4}",
                reading.items.len()
            );
        }
    }
}

fn copied(sample: &Snapshot, copies: usize) -> Snapshot {
    let mut reading = sample.clone();
    reading.items.clear();
    for copy in 0..copies {
        for (key, item) in &sample.items {
            reading
                .items
                .insert(format!("{key}{copy:04}"), item.clone());
        }
    }
    reading
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

#[cfg(target_os = "linux")]
fn reading() {
    let settings = Settings::plain(|| TAKEN_AT.to_string());
    let collector = match vigil_containers::Containers.collector(&settings) {
        Ok(collector) => collector,
        Err(why) => {
            println!("containers: {why}");
            return;
        }
    };

    let started = Instant::now();
    let Ok(first) = collector.collect() else {
        println!(
            "containers: nothing to read here: {:?}",
            collector.available()
        );
        return;
    };
    println!(
        "containers: first reading {:.2} ms, {} items, {} bytes as the baseline",
        started.elapsed().as_secs_f64() * 1000.0,
        first.items.len(),
        serde_json::to_string(&first).expect("serialises").len()
    );

    let started = Instant::now();
    for _ in 0..ROUNDS {
        let _ = collector.collect();
    }
    println!(
        "containers: {:.2} ms per reading over {ROUNDS}",
        started.elapsed().as_secs_f64() * 1000.0 / f64::from(ROUNDS)
    );
}

#[cfg(not(target_os = "linux"))]
fn reading() {
    println!("containers: this reading is taken on Linux; nothing to measure here");
}

fn drawing() {
    let reading = vigil_containers::fixture::containers();
    let room = Room::of(160);

    println!("pane          rows ms  cells ms (a window of {WINDOW})");
    for pane in vigil_containers::WhatRunsInContainers.panes() {
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
