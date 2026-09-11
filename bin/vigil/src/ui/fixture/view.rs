use vigil_model::Severity;

use super::accounts::accounts;
use super::agent::{agent, collector};
use super::findings::finding;
use super::host::host;
use super::programs::{launches, processes};
use super::sockets::snapshot;
use super::startup::persistence;
use super::{Reading, Status, View};

pub fn view() -> View {
    let mut view = View {
        socket_path: "/run/vigil/vigil.sock".into(),
        status: Some(Status {
            host: host(),
            agent: agent(),
            sent_at: "2026-09-09T09:00:01.000Z".into(),
        }),
        readings: Default::default(),
        found: Default::default(),
        trouble: None,
    };
    view.readings.put("ports", Reading::Taken(snapshot()));
    view.readings.put("users", Reading::Taken(accounts()));
    view.readings.put("processes", Reading::Taken(processes()));
    view.readings
        .put("persistence", Reading::Taken(persistence()));
    view.found.findings = vec![
        finding("A new listening port on 0.0.0.0:4444", Severity::Critical),
        finding("A user logged in from a new address", Severity::Low),
        finding("A listening port closed", Severity::Low),
    ];
    view.found.capacity = 500;
    view
}

pub fn view_with_launches() -> View {
    let mut view = view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors.pop();
        status
            .agent
            .collectors
            .push(collector("launches", 15, 2, 0));
    }
    view.readings.put("launches", Reading::Taken(launches()));
    view
}
