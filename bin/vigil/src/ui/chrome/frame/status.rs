use crate::ui::View;

pub(super) fn status(view: &View, width: u16) -> String {
    let Some(status) = &view.status else {
        return fitted(
            &[match &view.trouble {
                Some(trouble) => trouble.headline(),
                None => format!("waiting for the agent at {}", view.socket_path),
            }],
            width,
        );
    };

    let collectors = &status.agent.collectors;
    let unwell = collectors
        .iter()
        .filter(|collector| collector.state.is_trouble())
        .count();
    let off = collectors
        .iter()
        .filter(|collector| collector.state == vigil_model::CollectorState::Off)
        .count();
    let by_severity = match status.agent.findings.by_severity.is_empty() {
        true => String::new(),
        false => format!(
            " ({})",
            status
                .agent
                .findings
                .by_severity
                .iter()
                .map(|(severity, count)| format!("{count} {severity}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };

    let mut parts = Vec::new();
    if let Some(trouble) = view.stale() {
        parts.push(format!("NOT ANSWERING · {}", trouble.headline()));
    }
    let health = match (unwell, off) {
        (0, 0) => "all reading".to_string(),
        (0, off) => format!("{} reading, {off} off", collectors.len() - off),
        (unwell, 0) => format!("{unwell} not reading everything"),
        (unwell, off) => format!("{unwell} not reading everything, {off} off"),
    };
    parts.push(format!("{} collector(s), {health}", collectors.len()));
    parts.push(format!(
        "{} finding(s){by_severity}",
        status.agent.findings.retained
    ));
    if view.stale().is_none() {
        parts.push(format!("read every {}s", status.agent.interval_seconds));
    }

    fitted(&parts, width)
}

fn fitted(parts: &[String], width: u16) -> String {
    let room = (width as usize).saturating_sub(1);
    let mut line = String::new();
    for part in parts {
        let next = match line.is_empty() {
            true => part.clone(),
            false => format!("{line} · {part}"),
        };
        if next.chars().count() > room {
            break;
        }
        line = next;
    }
    if line.is_empty() {
        line = parts
            .first()
            .map(|part| part.chars().take(room).collect())
            .unwrap_or_default();
    }
    format!(" {line}")
}
