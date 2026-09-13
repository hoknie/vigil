use vigil_model::Severity;

use super::agent::{agent, collector};
use super::files::files;
use super::findings::finding;
use super::host::host;
use super::launches::launches;
use super::programs::processes;
use super::resources::resources;
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
    view.readings
        .put("ports", Reading::Taken(vigil_ports::fixture::ports()));
    view.readings
        .put("users", Reading::Taken(vigil_users::fixture::users()));
    view.readings.put("processes", Reading::Taken(processes()));
    view.readings
        .put("persistence", Reading::Taken(persistence()));
    view.readings.put(
        "firewall",
        Reading::Taken(vigil_firewall::fixture::firewall()),
    );
    view.readings.put("resources", Reading::Taken(resources()));
    view.readings.put(
        "containers",
        Reading::Taken(vigil_containers::fixture::containers()),
    );
    view.readings.put("files", Reading::Taken(files()));
    view.found.findings = vec![
        finding("A new listening port on 0.0.0.0:4444", Severity::Critical),
        finding("A user logged in from a new address", Severity::Low),
        finding("A listening port closed", Severity::Low),
    ];
    view.found.capacity = 500;
    view
}

pub fn view_with_trouble() -> View {
    let mut view = view();
    if let Some(status) = view.status.as_mut() {
        status.agent = super::answers::watching();
    }
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
