use vigil_collect::Health;
use vigil_model::Snapshot;

use crate::modules::every_seconds_of;
use crate::socket::State;
use crate::types::{Reading, Startup};

use super::host;
use super::host::mac;

pub fn period(collector: &str) -> u32 {
    every_seconds_of(collector)
        .unwrap_or_else(|| panic!("{collector} is not a collector this build ships"))
}

pub fn state() -> State {
    told(false, false, false)
}

pub fn state_that_may_kill() -> State {
    told(true, false, false)
}

pub fn state_that_may_change() -> State {
    told(false, true, false)
}

pub fn state_that_may_control() -> State {
    told(false, false, true)
}

pub fn mac_that_may_change_and_control() -> State {
    let mut startup = startup(false, true, true);
    startup.host = mac();
    State::new(
        startup,
        &[("network", Health::Ok)],
        &["ndjson".to_string()],
        &[],
    )
}

fn told(
    killing_from_the_console: bool,
    accounts_from_the_console: bool,
    units_from_the_console: bool,
) -> State {
    State::new(
        startup(
            killing_from_the_console,
            accounts_from_the_console,
            units_from_the_console,
        ),
        &[("network", Health::Ok)],
        &["ndjson".to_string()],
        &[],
    )
}

fn startup(
    killing_from_the_console: bool,
    accounts_from_the_console: bool,
    units_from_the_console: bool,
) -> Startup {
    Startup {
        configuration_path: "/etc/vigil/vigil.yaml".to_string(),
        host: host(),
        started_at: "2026-09-09T08:00:00.000Z".into(),
        interval_seconds: 30,
        periods: [("network".to_string(), period("network"))]
            .into_iter()
            .collect(),
        killing_from_the_console,
        accounts_from_the_console,
        units_from_the_console,
    }
}

pub fn reading(snapshot: Snapshot) -> Reading {
    reading_of("network", snapshot)
}

pub fn reading_of(collector: &'static str, snapshot: Snapshot) -> Reading {
    Reading {
        collector,
        at: "2026-09-09T09:00:00.000Z".into(),
        duration_ms: 5,
        every_seconds: period(collector),
        next_run_at: "2026-09-09T09:00:30.000Z".into(),
        skipped: 0,
        snapshot,
    }
}
