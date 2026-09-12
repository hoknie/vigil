use vigil_model::Change;

use super::{
    ResourceLimits, account_rules, container_rules, file_rules, firewall_rules, launch_rules,
    listening_port_rules, persistence_rules, process_rules, resource_rules,
};
use crate::{RuleContext, RuleSet};

fn findings_for(rules: RuleSet, changes: &[Change]) -> Vec<(String, String)> {
    let mut minted = 0;
    let mut mint = || {
        minted += 1;
        format!("event-{minted}")
    };
    let mut ctx = RuleContext {
        now: "2026-09-09T12:04:02.104Z".into(),
        mint_event_id: &mut mint,
    };

    rules
        .judge(changes, &mut ctx)
        .into_iter()
        .map(|finding| (finding.rule.unwrap_or_default(), finding.kind.to_string()))
        .collect()
}

pub(super) fn ports(change: &Change) -> Vec<(String, String)> {
    findings_for(listening_port_rules(), std::slice::from_ref(change))
}

pub(super) fn ports_tick(changes: &[Change]) -> Vec<(String, String)> {
    findings_for(listening_port_rules(), changes)
}

pub(super) fn accounts(change: &Change) -> Vec<(String, String)> {
    findings_for(account_rules(), std::slice::from_ref(change))
}

pub(super) fn persistence(change: &Change) -> Vec<(String, String)> {
    findings_for(persistence_rules(), std::slice::from_ref(change))
}

pub(super) fn processes(change: &Change) -> Vec<(String, String)> {
    findings_for(process_rules(), std::slice::from_ref(change))
}

pub(super) fn launches(change: &Change) -> Vec<(String, String)> {
    findings_for(launch_rules(), std::slice::from_ref(change))
}

pub(super) fn firewall(change: &Change) -> Vec<(String, String)> {
    findings_for(firewall_rules(), std::slice::from_ref(change))
}

pub(super) fn firewall_tick(changes: &[Change]) -> Vec<(String, String)> {
    findings_for(firewall_rules(), changes)
}

pub(super) fn containers(change: &Change) -> Vec<(String, String)> {
    findings_for(container_rules(), std::slice::from_ref(change))
}

pub(super) fn files(change: &Change) -> Vec<(String, String)> {
    findings_for(file_rules(), std::slice::from_ref(change))
}

pub(super) fn resources(change: &Change) -> Vec<(String, String)> {
    findings_for(
        resource_rules(ResourceLimits::default()),
        std::slice::from_ref(change),
    )
}

pub(super) fn resources_tick(changes: &[Change]) -> Vec<(String, String)> {
    findings_for(resource_rules(ResourceLimits::default()), changes)
}
