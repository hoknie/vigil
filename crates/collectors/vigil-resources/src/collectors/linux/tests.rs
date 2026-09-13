use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use vigil_model::Snapshot;

use super::files::{BOOT_ID, MEMINFO, MOUNTS, UPTIME};
use super::*;
use crate::parsers::BOOT;

const MEMORY: &str = "memory|summary";

const ONE_BOOT: &str = "1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e31";

const ANOTHER_BOOT: &str = "7d3b9a10-2e45-4c81-b6f7-9a0c1d2e3f40";

const MEMORY_OF_A_SMALL_HOST: &str = "MemTotal:        8039152 kB\n\
     MemFree:          311104 kB\n\
     MemAvailable:    5120884 kB\n\
     SwapTotal:       1048572 kB\n\
     SwapFree:        1048572 kB\n";

const FIRST_UPTIME: i64 = 128_142;

struct Bench {
    directory: PathBuf,
    wall_clock: Arc<AtomicI64>,
    uptime: AtomicI64,
}

impl Bench {
    fn new(named: &str) -> Bench {
        let directory = std::env::temp_dir().join(format!(
            "vigil-resources-{named}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(directory.join("sys/kernel/random")).expect("a bench to read from");
        fs::create_dir_all(directory.join("self")).expect("a bench to read from");

        let bench = Bench {
            directory,
            wall_clock: Arc::new(AtomicI64::new(1_757_547_342)),
            uptime: AtomicI64::new(FIRST_UPTIME),
        };
        bench.write(BOOT_ID, &format!("{ONE_BOOT}\n"));
        bench.write(MEMINFO, MEMORY_OF_A_SMALL_HOST);
        bench.uptime_reads(FIRST_UPTIME);
        bench.mounts_are(&bench.directory.display().to_string(), "ext4");
        bench
    }

    fn mounts_are(&self, target: &str, kind: &str) {
        self.write(
            MOUNTS,
            &format!("/dev/vigil-bench {target} {kind} rw,relatime 0 0\n"),
        );
    }

    fn uptime_reads(&self, seconds: i64) {
        self.write(UPTIME, &format!("{seconds}.56 1005819.02\n"));
    }

    fn write(&self, file: &str, text: &str) {
        fs::write(self.directory.join(file), text).expect("writes");
    }

    fn remove(&self, file: &str) {
        fs::remove_file(self.directory.join(file)).expect("removes");
    }

    fn collector(&self) -> ResourcesCollector {
        let wall_clock = Arc::clone(&self.wall_clock);

        ResourcesCollector::with_sources(
            || "2026-09-11T12:00:00.000Z".to_string(),
            &self.directory,
            move || Some(wall_clock.load(Ordering::Relaxed)),
        )
    }

    fn a_second_passes(&self, seconds: i64) {
        self.wall_clock.fetch_add(seconds, Ordering::Relaxed);
        self.uptime_reads(self.uptime.fetch_add(seconds, Ordering::Relaxed) + seconds);
    }

    fn the_two_clocks_disagree_by(&self, seconds: i64) {
        self.uptime_reads(self.uptime.load(Ordering::Relaxed) + seconds);
    }

    fn the_clock_is_stepped(&self, seconds: i64) {
        self.wall_clock.fetch_add(seconds, Ordering::Relaxed);
    }

    fn it_reboots(&self) {
        self.write(BOOT_ID, &format!("{ANOTHER_BOOT}\n"));
        self.uptime.store(4, Ordering::Relaxed);
        self.uptime_reads(4);
    }
}

impl Drop for Bench {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn booted_at(reading: &Snapshot) -> i64 {
    reading.items[BOOT]["booted_at"]
        .as_i64()
        .expect("the moment this host booted")
}

fn said(health: &Health) -> String {
    match health {
        Health::Ok => String::new(),
        Health::Degraded(detail) | Health::Unavailable(detail) => detail.clone(),
    }
}

#[test]
fn a_host_that_did_not_move_reads_the_same_on_every_tick() {
    let bench = Bench::new("steady");
    let collector = bench.collector();

    let first = collector.collect().expect("reads");
    let mut readings = Vec::new();
    for round in 0..30 {
        bench.a_second_passes(30);
        bench.the_two_clocks_disagree_by(round % 3 - 1);
        readings.push(collector.collect().expect("reads"));
    }

    assert_eq!(collector.available(), Health::Ok);
    for later in &readings {
        assert_eq!(
            first.items, later.items,
            "the seconds since boot and the free bytes are read past rather than recorded, and \
             two whole-second clocks disagreeing by one is not the host changing"
        );
    }
}

#[test]
fn a_filesystem_this_host_mounts_is_read_as_a_step_and_never_as_the_bytes_left_on_it() {
    let bench = Bench::new("filesystems");
    let collector = bench.collector();
    let key = format!("fs|{}", bench.directory.display());

    let reading = collector.collect().expect("reads");
    let row = reading.items.get(&key).expect("the bench filesystem");

    assert_eq!(collector.available(), Health::Ok);
    assert_eq!(row["type"], "ext4");
    assert_eq!(row["read_only"], false);
    assert!(row["total_bytes"].as_u64().expect("a size") > 0);
    let step = row["free_percent_step"].as_u64().expect("a step");
    assert!(step.is_multiple_of(5) && step <= 100, "{step}");
    assert!(
        row.get("free_bytes").is_none(),
        "the bytes left on a filesystem anything is writing to differ between any two readings"
    );
}

#[test]
fn a_filesystem_on_the_other_side_of_a_network_is_never_asked_how_full_it_is() {
    let bench = Bench::new("network");
    let collector = bench.collector();

    bench.mounts_are("/mnt/shared", "nfs4");
    let reading = collector.collect().expect("reads");

    assert_eq!(collector.available(), Health::Ok);
    assert!(
        reading.items.keys().all(|key| !key.starts_with("fs|")),
        "statvfs on a mount whose server is gone waits for it, and a reading that waits \
         forever stops the watch behind it"
    );
}

#[test]
fn a_mount_point_that_would_not_answer_is_a_complaint_and_not_a_filesystem_with_room() {
    let bench = Bench::new("unanswered");
    let collector = bench.collector();

    bench.mounts_are("/there/is/no/such/path", "ext4");
    let health = collector.available();

    assert!(matches!(health, Health::Degraded(_)), "{health:?}");
    assert!(
        said(&health).contains("/there/is/no/such/path"),
        "{}",
        said(&health)
    );
    assert!(
        collector
            .collect()
            .expect("the rest of the reading is still taken")
            .items
            .keys()
            .all(|key| !key.starts_with("fs|"))
    );
}

#[test]
fn a_host_that_rebooted_says_so_in_the_boot_it_is_running_and_in_the_moment_it_booted() {
    let bench = Bench::new("reboot");
    let collector = bench.collector();

    let before = collector.collect().expect("reads");
    bench.a_second_passes(60);
    bench.it_reboots();
    let after = collector.collect().expect("reads");

    assert_eq!(before.items[BOOT]["boot_id"], ONE_BOOT);
    assert_eq!(after.items[BOOT]["boot_id"], ANOTHER_BOOT);
    assert!(
        booted_at(&after) > booted_at(&before),
        "a host that booted again booted later than it booted before"
    );
}

#[test]
fn a_clock_that_was_stepped_moves_the_moment_this_host_booted_and_leaves_the_boot_alone() {
    let bench = Bench::new("skew");
    let collector = bench.collector();

    let before = collector.collect().expect("reads");
    bench.a_second_passes(30);
    bench.the_clock_is_stepped(-3_600);
    let after = collector.collect().expect("reads");

    assert_eq!(
        before.items[BOOT]["boot_id"], after.items[BOOT]["boot_id"],
        "nothing rebooted: the same kernel is running and it says so"
    );
    assert_eq!(booted_at(&before) - booted_at(&after), 3_600);
}

#[test]
fn the_reading_a_restart_left_behind_is_the_one_the_next_reading_is_compared_with() {
    let bench = Bench::new("restart");
    let before = bench.collector().collect().expect("reads");

    let after_the_restart = bench.collector();
    after_the_restart.restore(&before);
    bench.a_second_passes(1);
    let first_after = after_the_restart.collect().expect("reads");

    assert_eq!(
        before.items, first_after.items,
        "a daemon that came back is not a host that changed while it was away"
    );
}

#[test]
fn the_files_this_collector_reads_fail_in_different_ways_and_none_of_them_is_an_empty_host() {
    let bench = Bench::new("refusals");
    let collector = bench.collector();

    bench.remove(MEMINFO);
    let without_memory = collector.available();
    assert!(
        matches!(without_memory, Health::Degraded(_)),
        "{without_memory:?}"
    );
    assert!(
        !collector
            .collect()
            .expect("still reads")
            .items
            .contains_key(MEMORY),
        "a host whose memory could not be read carries no row saying it has none"
    );

    bench.write(BOOT_ID, "not a boot identifier\n");
    let without_boot_id = collector.available();
    assert!(said(&without_boot_id).contains("shape this build does not know"));
    assert_eq!(
        collector.collect().expect("still reads").items[BOOT]["readable"],
        false
    );

    bench.remove(UPTIME);
    bench.remove(MOUNTS);
    let nothing = collector.available();
    assert!(matches!(nothing, Health::Unavailable(_)), "{nothing:?}");
    assert!(
        collector.collect().is_err(),
        "a reading with nothing in it would be stored as a host with nothing on it"
    );

    assert_ne!(said(&without_memory), said(&without_boot_id));
    assert_ne!(said(&without_boot_id), said(&nothing));
}
