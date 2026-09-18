use std::path::Path;

use super::store::{baselines, health, history};
use super::{console, following, greeting, outgoing, policy, reporters, schedule, watches};
use crate::budget::Meter;
use crate::helpers::{absolute, agent_finding, rfc3339};
use crate::loops::Round;
use crate::socket::{Shared, State, switched_off_reasons};
use crate::types::{Delivery, Startup};
use crate::{config, identity};

pub fn run(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let stamp = config::Stamp::of(config_path);
    let config = config::load(config_path)?;
    let state_dir = Path::new(&config.state_dir);

    let host = identity::describe(state_dir)?;
    let reporters = reporters::build(&config.reporters)?;
    let buffers = outgoing::open(state_dir, &reporters)?;
    let (mut watches, switched_off) = watches::open(&config)?;
    let store = history::open(state_dir)?;
    let schedule = schedule::of(&config, &watches, &host.host_id);

    let shared = Shared::new(State::new(
        Startup {
            host: host.clone(),
            configuration_path: absolute::of(config_path),
            started_at: rfc3339::now(),
            interval_seconds: config.every_seconds_by_default(),
            periods: schedule
                .periods()
                .into_iter()
                .map(|(name, every_seconds)| (name.to_string(), every_seconds))
                .collect(),
            killing_from_the_console: config.killing.from_the_console,
            accounts_from_the_console: config.accounts.from_the_console,
            units_from_the_console: config.units.from_the_console,
        },
        &watches::healths(&watches),
        &reporters::names(&reporters),
        &switched_off_reasons(&config, &switched_off),
    ));

    let greeting = greeting::Greeting {
        config_path,
        config: &config,
        host: &host,
        watches: &watches,
        switched_off: &switched_off,
        reporters: reporters.len(),
        damaged: store.damaged_on_open(),
    };
    greeting.say();
    let at_start = greeting.findings();

    let delivery = Delivery::new(host, reporters, buffers, shared.clone());
    let policy = policy::of(&config);

    history::prune(&store, config.retention_days);
    for line in health::renamed(&store, &rfc3339::now()) {
        eprintln!("{line}");
    }
    history::recall(&store, &shared, &policy);

    let mut opening = at_start;
    let standing = health::standing(&store, &watches);
    for (reporter, lines) in delivery.damaged() {
        opening.push(agent_finding::store_damaged(
            &format!("outgoing/{reporter}"),
            &format!("the outgoing buffer for {reporter}"),
            lines,
        ));
    }
    opening.extend(delivery.replay());

    baselines::forget(&store, &switched_off);
    baselines::restore(&store, &mut watches);

    console::listen(&config, &schedule, &shared)?;
    let followed = following::of(config_path, stamp, &config, &watches);
    let silences = config::Silences::of(config_path, &config);

    Round {
        watches,
        store,
        policy,
        delivery,
        shared,
        schedule,
        meter: Meter::default(),
        opening,
        standing,
        followed,
        silences,
    }
    .run()
}
