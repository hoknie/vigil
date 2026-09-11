use ratatui::text::{Line, Span};

use super::room::ROOM_FOR_TWO_COLUMNS;
use crate::ui::helpers::layout::column;
use crate::ui::helpers::layout::field;
use crate::ui::helpers::layout::section;
use crate::ui::{Look, Report, View};

pub(super) fn identity(report: &mut Report, view: &View, look: Look, width: usize) {
    let Some(status) = &view.status else {
        return;
    };
    let host = &status.host;
    let agent = &status.agent;

    let about_the_host = vec![
        ("hostname", host.hostname.clone()),
        (
            "system",
            format!(
                "{} {} · {} · kernel {}",
                host.os.distro, host.os.version, host.os.arch, host.os.kernel
            ),
        ),
        ("host id", host.host_id.clone()),
        ("install id", host.install_id.clone()),
        ("boot id", host.boot_id.clone()),
    ];
    let about_the_agent = vec![
        ("version", agent.version.clone()),
        ("started", format!("{} (agent's clock)", agent.started_at)),
        (
            "reads",
            format!(
                "every {} seconds, unless a collector names its own period",
                agent.interval_seconds
            ),
        ),
        ("costs", cost(agent)),
        ("answered", status.sent_at.clone()),
        (
            "findings",
            format!(
                "{} of {} on the findings screen",
                agent.findings.retained, agent.findings.capacity,
            ),
        ),
        ("history", history(agent)),
    ];

    if width < ROOM_FOR_TWO_COLUMNS as usize {
        report.push(section::rule(look, "HOST", width));
        for (name, value) in &about_the_host {
            report.push(field::one(look, name, value, 12, width));
        }
        report.blank();
        report.push(section::rule(look, "AGENT", width));
        for (name, value) in &about_the_agent {
            report.push(field::one(look, name, value, 12, width));
        }
        report.blank();
        return;
    }

    report.push(section::rule(
        look,
        "THIS HOST, AND THE AGENT WATCHING IT",
        width,
    ));
    let half = width / 2;
    for index in 0..about_the_host.len().max(about_the_agent.len()) {
        let mut spans = Vec::new();
        match about_the_host.get(index) {
            Some((name, value)) => {
                spans.extend(field::one(look, name, value, 12, half).spans);
            }
            None => spans.push(Span::raw(column::fit("", half))),
        }
        if let Some((name, value)) = about_the_agent.get(index) {
            spans.extend(field::one(look, name, value, 12, width - half).spans);
        }
        report.push(Line::from(spans));
    }
    report.blank();
}

fn history(agent: &vigil_model::AgentStatus) -> String {
    let total = agent.findings.total;
    match (total, agent.findings.dropped) {
        (0, _) => "nothing open has been recorded yet".to_string(),
        (total, 0) => format!("{total} open, and the screen holds them all"),
        (total, elsewhere) => format!("{total} open, {elsewhere} of them in the journal only"),
    }
}

fn cost(agent: &vigil_model::AgentStatus) -> String {
    let duty = match agent.budget.duty_percent {
        Some(percent) => format!("{percent:.2}% of one core"),
        None => "a share of one core this platform does not measure".to_string(),
    };
    let resident = match agent.budget.resident_kb {
        Some(kilobytes) => format!("{} MB resident", kilobytes / 1024),
        None => "memory this platform does not measure".to_string(),
    };
    format!("{duty} · {resident}")
}
