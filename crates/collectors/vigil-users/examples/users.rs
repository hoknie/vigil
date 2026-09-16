use std::time::Instant;

#[cfg(target_os = "linux")]
use vigil_module::{Module, Settings};
use vigil_view::{Room, Section, Showing, listed};

#[cfg(target_os = "linux")]
const TAKEN_AT: &str = "2026-09-13T12:00:00.000Z";
const ROUNDS: u32 = 200;
const WINDOW: usize = 40;
const TYPED: &str = "deploy";

fn main() {
    reading();
    drawing();
}

#[cfg(target_os = "linux")]
fn reading() {
    let settings = Settings::plain(|| TAKEN_AT.to_string());
    let collector = match vigil_users::Users.collector(&settings) {
        Ok(collector) => collector,
        Err(why) => {
            println!("users: {why}");
            return;
        }
    };

    let started = Instant::now();
    let Ok(first) = collector.collect() else {
        println!("users: nothing to read here: {:?}", collector.available());
        return;
    };
    println!(
        "users: first reading {:.2} ms, {} items, {} bytes as the baseline",
        started.elapsed().as_secs_f64() * 1000.0,
        first.items.len(),
        serde_json::to_string(&first).expect("serialises").len()
    );

    let started = Instant::now();
    for _ in 0..ROUNDS {
        let _ = collector.collect();
    }
    println!(
        "users: {:.2} ms per reading over {ROUNDS}",
        started.elapsed().as_secs_f64() * 1000.0 / f64::from(ROUNDS)
    );
}

#[cfg(not(target_os = "linux"))]
fn reading() {
    println!("users: the reading walks a Linux /etc; nothing to measure here");
}

fn drawing() {
    let reading = vigil_users::fixture::users();
    let room = Room::of(160);

    println!(
        "pane          rows ms  cells ms (a window of {WINDOW})  keystroke by rows ms  index ms  \
         keystroke by index ms (typing {TYPED:?})"
    );
    for pane in vigil_users::WhoCanLogIn.panes() {
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

        let keystrokes = f64::from(ROUNDS) * TYPED.len() as f64;
        let started = Instant::now();
        for _ in 0..ROUNDS {
            for end in 1..=TYPED.len() {
                let _ = pane.rows(&reading, &Showing::searching(&TYPED[..end]));
            }
        }
        let by_rows = started.elapsed().as_secs_f64() * 1_000.0 / keystrokes;

        let started = Instant::now();
        let mut index = None;
        for _ in 0..ROUNDS {
            index = pane.index(&reading, &Showing::default());
        }
        let building = started.elapsed().as_secs_f64() * 1_000.0 / f64::from(ROUNDS);
        let Some(index) = index else {
            println!(
                "{:<12}  {listing:>7.4}  {cells:>8.4}  {by_rows:>20.4}  no index",
                pane.name()
            );
            continue;
        };

        let started = Instant::now();
        for _ in 0..ROUNDS {
            let mut within: Option<Vec<usize>> = None;
            for end in 1..=TYPED.len() {
                let showing = Showing::searching(&TYPED[..end]);
                let (found, _) =
                    listed(pane.as_ref(), &reading, &showing, &index, within.as_deref());
                within = Some(found);
            }
        }
        let by_index = started.elapsed().as_secs_f64() * 1_000.0 / keystrokes;

        let started = Instant::now();
        for _ in 0..ROUNDS {
            for end in 1..=TYPED.len() {
                let showing = Showing::searching(&TYPED[..end]);
                let _ = pane.tally(&reading, &showing, rows.len());
            }
        }
        let footer_by_reading = started.elapsed().as_secs_f64() * 1_000.0 / keystrokes;

        let started = Instant::now();
        let mut counts = None;
        for _ in 0..ROUNDS {
            counts = pane.counts(&reading, &Showing::default());
        }
        let counting = started.elapsed().as_secs_f64() * 1_000.0 / f64::from(ROUNDS);
        let counts = counts.unwrap_or_default();

        let started = Instant::now();
        for _ in 0..ROUNDS {
            for end in 1..=TYPED.len() {
                let showing = Showing::searching(&TYPED[..end]);
                let _ =
                    pane.tally_listed(&reading, &showing, &vigil_view::Rows::built(&rows), &counts);
            }
        }
        let footer_by_counts = started.elapsed().as_secs_f64() * 1_000.0 / keystrokes;
        println!(
            "{:<12}  footer a keystroke: by the reading {footer_by_reading:.4} ms, by the counts \
             {footer_by_counts:.4} ms; counting once {counting:.4} ms",
            pane.name()
        );

        println!(
            "{:<12}  {listing:>7.4}  {cells:>8.4}  {by_rows:>20.4}  {building:>8.4}  {by_index:>21.4}",
            pane.name()
        );
    }
}
