use vigil_model::{ControlReport, ControlTarget, Controlled};

pub(super) fn headline(report: &ControlReport) -> String {
    format!(
        "{} OF {} {}(S)",
        report.done(),
        report.controlled.len(),
        match report.target {
            ControlTarget::Unit => "UNIT",
            ControlTarget::Cron => "CRON JOB",
        }
    )
}

pub(super) fn said(report: &ControlReport) -> Vec<String> {
    let mut lines = vec![
        format!("{} at {}", report.controlling.said(), report.acted_at),
        String::new(),
    ];
    for one in &report.controlled {
        lines.push(row(one));
    }
    lines.push(String::new());
    lines.push(match (report.refused(), report.target) {
        (0, ControlTarget::Unit) => "The next reading says what this host starts by itself \
                                     now. Nothing here is a promise that the program has \
                                     already stopped."
            .to_string(),
        (0, ControlTarget::Cron) => "The file is written. The next reading says what cron has \
                                     left to run; a job already running is not stopped by a \
                                     hash in front of its line."
            .to_string(),
        (refused, _) => format!(
            "{refused} of them the agent did not touch, for the reason written beside each."
        ),
    });
    lines
}

fn row(one: &Controlled) -> String {
    let mark = match one.done {
        true => "done",
        false => "left",
    };
    match &one.object {
        Some(object) if object != &one.key => {
            format!("  [{mark}] {} — {object} — {}", one.key, one.said)
        }
        _ => format!("  [{mark}] {} — {}", one.key, one.said),
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::Controlling;

    use super::*;

    fn report(
        target: ControlTarget,
        controlling: Controlling,
        controlled: Vec<Controlled>,
    ) -> ControlReport {
        ControlReport {
            target,
            controlling,
            acted_at: "2026-09-16T10:00:00.000Z".into(),
            controlled,
        }
    }

    #[test]
    fn the_sheet_names_every_unit_the_agent_left_alone_and_why() {
        let lines = said(&report(
            ControlTarget::Unit,
            Controlling::Stop,
            vec![
                Controlled::done(
                    "unit|nginx.service",
                    Some("nginx.service".into()),
                    "/usr/bin/systemctl stop nginx.service",
                ),
                Controlled::refused(
                    "unit|vigild.service",
                    "vigild.service is a part of this agent",
                ),
            ],
        ));
        let page = lines.join("\n");

        assert!(page.contains("unit|vigild.service"), "{page}");
        assert!(page.contains("part of this agent"), "{page}");
        assert!(
            page.contains("1 of them the agent did not touch"),
            "a count of what worked with no count of what did not is the half of this sheet \
             an operator would have to work out for themselves: {page}"
        );
    }

    #[test]
    fn a_sheet_about_cron_counts_cron_jobs_and_says_what_a_hash_does_not_do() {
        let done = report(
            ControlTarget::Cron,
            Controlling::Comment,
            vec![Controlled::done(
                "cron|/etc/crontab|root|/usr/local/bin/agent",
                Some("/etc/crontab".into()),
                "/etc/crontab written",
            )],
        );

        assert_eq!(headline(&done), "1 OF 1 CRON JOB(S)");
        let page = said(&done).join("\n");
        assert!(
            page.contains("a job already running is not stopped"),
            "a reader who commented out a line and watched the process keep going has to \
             have been told: {page}"
        );
        assert!(!page.contains("systemctl"), "{page}");
    }
}
