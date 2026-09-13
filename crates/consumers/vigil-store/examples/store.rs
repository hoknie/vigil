use std::time::Instant;

use serde_json::json;
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};
use vigil_store::{FileStore, Limits, Outgoing};

const ROUNDS: usize = 20;
const BATCHES: &[usize] = &[1, 10, 100];
const AT: &str = "2026-09-11T12:00:00.000Z";

fn main() {
    if let Ok(count) = std::env::var("VIGIL_HOLD") {
        hold(count.parse().unwrap_or(0));
        return;
    }

    let rounds = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(ROUNDS)
        .max(1);

    let limits = Limits::outgoing();
    let one = serde_json::to_string(&finding(0))
        .expect("serialises")
        .len()
        + 1;
    println!(
        "outgoing buffer: ceiling {} finding(s) / {} kB · one finding {one} bytes on the wire",
        limits.findings,
        limits.journal_bytes / 1024
    );

    println!();
    println!("batch  keep ms  delivered ms  bytes held");
    for batch in BATCHES {
        let path = scratch(&format!("keep-{batch}"));
        let mut buffer = Outgoing::open("cost", path.clone(), limits).expect("opens");
        let findings: Vec<Finding> = (0..*batch).map(finding).collect();

        let started = Instant::now();
        for _ in 0..rounds {
            buffer.keep(&findings).expect("keeps");
        }
        let keeping = started.elapsed().as_secs_f64() * 1_000.0 / rounds as f64;

        let held = buffer.held().bytes.held;
        let started = Instant::now();
        for _ in 0..rounds {
            buffer.delivered(*batch).expect("hands over");
        }
        let handing = started.elapsed().as_secs_f64() * 1_000.0 / rounds as f64;

        println!("{batch:>5}  {keeping:>7.3}  {handing:>12.3}  {held:>10}");
        let _ = std::fs::remove_file(&path);
    }

    let full = scratch("full");
    let mut buffer = Outgoing::open("cost", full.clone(), limits).expect("opens");
    let findings: Vec<Finding> = (0..limits.findings).map(finding).collect();
    buffer.keep(&findings).expect("fills it");
    let bytes = buffer.held().bytes.held;
    drop(buffer);

    let started = Instant::now();
    for _ in 0..rounds {
        let buffer = Outgoing::open("cost", full.clone(), limits).expect("opens");
        std::hint::black_box(buffer.held());
    }
    let opening = started.elapsed().as_secs_f64() * 1_000.0 / rounds as f64;

    let block: Vec<Finding> = (0..limits.findings / 10 + 1).map(finding).collect();
    let mut buffer = Outgoing::open("cost", full.clone(), limits).expect("opens");
    let started = Instant::now();
    for _ in 0..rounds {
        buffer.keep(&block).expect("keeps");
    }
    let at_the_ceiling = started.elapsed().as_secs_f64() * 1_000.0 / rounds as f64;

    let room = scratch("room");
    let mut buffer = Outgoing::open("cost", room.clone(), limits).expect("opens");
    let started = Instant::now();
    for _ in 0..rounds {
        buffer.keep(&block).expect("keeps");
        buffer.delivered(usize::MAX).expect("hands over");
    }
    let with_room = started.elapsed().as_secs_f64() * 1_000.0 / rounds as f64;

    println!();
    println!(
        "a buffer at its ceiling: {} finding(s), {} kB",
        limits.findings,
        bytes / 1024
    );
    println!("  replay on start            {opening:>8.3} ms");
    println!(
        "  {} more finding(s), compacting  {at_the_ceiling:>8.3} ms",
        block.len()
    );
    println!("  the same, with room for them  {with_room:>8.3} ms  (write plus the hand-over)");
    let _ = std::fs::remove_file(&full);
    let _ = std::fs::remove_file(&room);

    history(rounds);
}

fn history(rounds: usize) {
    let limits = Limits::default();
    let directory = std::env::temp_dir().join(format!("vigil-cost-{}-history", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(directory.join("findings")).expect("makes the directory");
    let lines: String = (0..limits.findings)
        .map(|index| {
            format!(
                "{}\n",
                serde_json::to_string(&finding(index)).expect("serialises")
            )
        })
        .collect();
    let bytes = lines.len();
    std::fs::write(directory.join("findings").join("journal.ndjson"), lines).expect("writes");

    let started = Instant::now();
    for _ in 0..rounds {
        let store = FileStore::open(&directory).expect("opens");
        std::hint::black_box(store.compactions());
    }
    let opening = started.elapsed().as_secs_f64() * 1_000.0 / rounds as f64;

    println!();
    println!(
        "the findings history at its ceiling: {} record(s), {} kB",
        limits.findings,
        bytes / 1024
    );
    println!("  read on start     {opening:>8.3} ms  (what every compaction used to pay again)");
    let _ = std::fs::remove_dir_all(&directory);
}

fn hold(count: usize) {
    let path = scratch(&format!("hold-{count}"));
    let _ = std::fs::remove_file(&path);
    {
        let mut buffer = Outgoing::open("cost", path.clone(), Limits::outgoing()).expect("opens");
        for chunk in 0..count / 50 {
            let block: Vec<Finding> = (0..50).map(|index| finding(chunk * 50 + index)).collect();
            buffer.keep(&block).expect("keeps");
        }
    }

    let buffer = Outgoing::open("cost", path.clone(), Limits::outgoing()).expect("reopens");
    println!(
        "holding {} finding(s), {} kB on disk",
        buffer.held().records.held,
        buffer.held().bytes.held / 1024
    );
    let _ = std::fs::remove_file(&path);
}

fn scratch(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("vigil-cost-{}-{name}.ndjson", std::process::id()))
}

fn finding(index: usize) -> Finding {
    Finding {
        event_id: format!("0199a1b2-c3d4-7e5f-8a9b-{index:012x}"),
        finding_key: format!("port.listen|tcp|0.0.0.0:{index}"),
        kind: Kind::Known(KnownKind::PortListenNew),
        severity: Severity::High,
        state: State::Open,
        observed_at: AT.to_string(),
        first_seen_at: AT.to_string(),
        occurrences: 1,
        title: format!("A program is listening on 0.0.0.0:{index} that was not there before"),
        subject: Subject {
            object: "socket".into(),
            key: json!({ "protocol": "tcp", "address": "0.0.0.0", "port": index }),
        },
        before: None,
        after: Some(json!({
            "protocol": "tcp", "address": "0.0.0.0", "port": index, "uid": 0, "user": "root",
            "process": {"exe": "/usr/sbin/nginx", "exe_deleted": false, "cmdline": "nginx: master process", "cmdline_redacted": false},
        })),
        evidence: vec![Evidence {
            kind: "note".into(),
            value: "the program holding the socket was started 4 s before this reading".into(),
        }],
        redacted: Vec::new(),
        rule: Some("new_listening_port".into()),
        labels: Default::default(),
    }
}
