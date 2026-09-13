#[cfg(target_os = "linux")]
fn main() {
    use std::time::Instant;
    use vigil_collect::Collector;

    fn peak_resident_kb() -> u64 {
        std::fs::read_to_string("/proc/self/status")
            .unwrap_or_default()
            .lines()
            .find(|line| line.starts_with("VmHWM:"))
            .and_then(|line| line.split_whitespace().nth(1)?.parse().ok())
            .unwrap_or(0)
    }

    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "processes".into());
    let rounds: u32 = std::env::args()
        .nth(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(500);

    let now = || "2026-09-09T12:00:00.000Z".to_string();
    let collector: Box<dyn Collector> = match name.as_str() {
        "persistence" => Box::new(vigil_collect::PersistenceCollector::new(now)),
        "processes" => Box::new(vigil_collect::ProcessesCollector::new(now)),
        "resources" => Box::new(vigil_collect::ResourcesCollector::new(now)),
        "files" => Box::new(vigil_collect::FilesCollector::new(
            now,
            &[
                "/etc/ssh/sshd_config".to_string(),
                "/etc/hosts".to_string(),
                "/etc/nsswitch.conf".to_string(),
            ],
            1024 * 1024,
        )),
        "launches" => Box::new(vigil_collect::LaunchesCollector::new(now, false)),
        other => {
            eprintln!("no collector named {other}");
            std::process::exit(2);
        }
    };

    let started = Instant::now();
    let first = match collector.collect() {
        Ok(first) => first,
        Err(refusal) => {
            println!("{name}: health {:?}", collector.available());
            println!("{name}: nothing to measure here: {refusal}");
            return;
        }
    };
    let first_took = started.elapsed();

    println!("{name}: health {:?}", collector.available());
    println!(
        "{name}: first reading {:.2} ms",
        first_took.as_secs_f64() * 1000.0
    );
    println!("{name}: {} items", first.items.len());
    println!(
        "{name}: {} bytes stored as the baseline",
        serde_json::to_string(&first).expect("serialises").len()
    );

    let before = peak_resident_kb();
    let started = Instant::now();
    for _ in 0..rounds {
        let _ = collector.collect();
    }
    let elapsed = started.elapsed();

    println!(
        "{name}: {:.2} ms per reading over {rounds}, peak RSS {} kB (+{} kB while reading)",
        elapsed.as_secs_f64() * 1000.0 / f64::from(rounds),
        peak_resident_kb(),
        peak_resident_kb().saturating_sub(before)
    );
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("the collectors read a Linux /proc; nothing to measure here");
}
