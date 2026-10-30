use vigil_model::{Change, Finding};

use crate::{RuleContext, RuleSet};

pub fn findings_for(rules: RuleSet, changes: &[Change]) -> Vec<(String, String)> {
    judged(rules, changes)
        .into_iter()
        .map(|finding| (finding.rule.unwrap_or_default(), finding.kind.to_string()))
        .collect()
}

pub fn programs_for(rules: RuleSet, changes: &[Change]) -> Vec<(String, Option<String>)> {
    judged(rules, changes)
        .into_iter()
        .map(|finding| {
            let program = finding.program().map(str::to_string);
            (finding.kind.to_string(), program)
        })
        .collect()
}

fn judged(rules: RuleSet, changes: &[Change]) -> Vec<Finding> {
    let mut minted = 0;
    let mut mint = || {
        minted += 1;
        format!("event-{minted}")
    };
    let mut ctx = RuleContext {
        now: "2026-09-09T12:04:02.104Z".into(),
        mint_event_id: &mut mint,
    };

    rules.judge(changes, &mut ctx)
}
