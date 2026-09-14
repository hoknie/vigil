use vigil_model::{ChangeReport, Changed};

pub(super) fn headline(report: &ChangeReport, left: &[(String, String)]) -> String {
    format!(
        "{} OF {} CHANGE(S)",
        report.done(),
        report.changed.len() + left.len()
    )
}

pub(super) fn said(report: &ChangeReport, left: &[(String, String)]) -> Vec<String> {
    let mut lines = vec![format!("at {}", report.acted_at), String::new()];
    for changed in &report.changed {
        lines.push(one(changed));
    }
    for (key, why) in left {
        lines.push(format!(
            "  [left] {key} — {why} (the console did not ask for it)"
        ));
    }
    lines.push(String::new());
    lines.push(match report.refused() + left.len() {
        0 => "The agent reads the accounts of this host again straight after a change: the \
              list shows this within a few seconds."
            .to_string(),
        refused => format!(
            "{refused} of them were not done, for the reason written beside each. The agent \
             reads the accounts again straight away, and the list shows the rest within a few \
             seconds."
        ),
    });
    lines
}

fn one(changed: &Changed) -> String {
    let mark = match changed.done {
        true => "done",
        false => "left",
    };
    format!("  [{mark}] {} — {}", changed.key, changed.said)
}
