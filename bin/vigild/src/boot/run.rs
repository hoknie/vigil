use std::path::Path;

use super::{
    baselines, console, greeting, health, history, outgoing, policy, reporters, schedule, watches,
};
use crate::budget::Meter;
use crate::helpers::{absolute, agent_finding, rfc3339};
use crate::loops::Round;
use crate::socket::{Shared, State};
use crate::types::{Delivery, Startup};
use crate::{config, identity};

pub fn run(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
        },
        &watches::healths(&watches),
        &reporters::names(&reporters),
        &switched_off,
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
    }
    .run()
}
