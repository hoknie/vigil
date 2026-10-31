use vigil_collect::Health;
use vigil_module::Module;
use vigil_store::Store;

use super::harness::{raised, snapshot, watching};
use crate::loops::Watch;
use crate::types::Said;

fn kinds(round: &crate::loops::Round) -> Vec<String> {
    raised(round)
        .iter()
        .map(|finding| finding.kind.as_str().to_string())
        .collect()
}

#[test]
fn a_new_baseline_raises_one_info_finding_about_the_agent_and_none_about_the_host() {
    let mut it = watching("baseline", Health::Ok, vec![snapshot(&[443, 4444])]);
    let mut said = Said::about([("network", "ok".to_string())]);

    it.round.read(0, &mut said);

    let findings = raised(&it.round);
    assert_eq!(kinds(&it.round), ["agent.baseline.ready"], "{findings:?}");
    assert_eq!(findings[0].finding_key, "agent.baseline|network");
    assert_eq!(findings[0].severity, vigil_model::Severity::Info);
    assert!(
        findings[0].evidence[0].value.starts_with("2 item(s)"),
        "{:?}",
        findings[0].evidence
    );
}

#[test]
fn a_baseline_restored_after_a_restart_is_not_announced_again() {
    let mut it = watching("restored", Health::Ok, vec![snapshot(&[443, 4444])]);
    it.round.watches[0].restore(snapshot(&[443, 4444]));
    let mut said = Said::about([("network", "ok".to_string())]);

    it.round.read(0, &mut said);

    assert!(
        !kinds(&it.round).contains(&"agent.baseline.ready".to_string()),
        "a restart continues watching; it does not learn the host again: {:?}",
        kinds(&it.round)
    );
}

#[test]
fn a_collector_switched_on_again_announces_its_baseline_as_a_repeat() {
    let mut it = watching(
        "again",
        Health::Ok,
        vec![snapshot(&[443]), snapshot(&[443])],
    );
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.read(0, &mut said);

    let rules = vigil_network::Network.rules(&vigil_module::Settings::plain(|| {
        "2026-09-09T12:00:00.000Z".to_string()
    }));
    it.round.watches[0] = Watch::new(Box::new(Again(it.collector.clone())), rules);
    it.round.read(0, &mut said);

    let kept = it
        .round
        .store
        .open_finding("agent.baseline|network", "agent.baseline.ready")
        .expect("readable")
        .expect("in the journal");
    assert_eq!(
        kept.occurrences, 2,
        "the same statement about the same collector, said again, is one finding said twice"
    );
}

struct Again(std::sync::Arc<super::harness::Scripted>);

impl vigil_collect::Collector for Again {
    fn name(&self) -> &'static str {
        self.0.name()
    }
    fn available(&self) -> Health {
        self.0.available()
    }
    fn collect(&self) -> Result<vigil_model::Snapshot, vigil_collect::CollectError> {
        self.0.collect()
    }
}

struct StandingDanger;

impl vigil_rules::Rule for StandingDanger {
    fn name(&self) -> &'static str {
        "standing_danger"
    }

    fn apply(
        &self,
        change: &vigil_model::Change,
        ctx: &mut vigil_rules::RuleContext<'_>,
    ) -> Option<vigil_model::Finding> {
        let vigil_model::Change::Added { key, after } = change else {
            return None;
        };
        if after["port"] != 4444 {
            return None;
        }
        vigil_network::Network
            .rules(&vigil_module::Settings::plain(|| {
                "2026-09-09T12:00:00.000Z".to_string()
            }))
            .judge(
                &[vigil_model::Change::Added {
                    key: key.clone(),
                    after: after.clone(),
                }],
                ctx,
            )
            .into_iter()
            .next()
    }

    fn judges_the_baseline(&self) -> bool {
        true
    }
}

#[test]
fn a_rule_judging_the_baseline_still_speaks_beside_the_baseline_finding() {
    let mut it = watching("standing", Health::Ok, vec![snapshot(&[443, 4444])]);
    it.round.watches[0] = Watch::new(
        Box::new(Again(it.collector.clone())),
        vigil_rules::RuleSet::new(Vec::new(), vec![Box::new(StandingDanger)]),
    );
    let mut said = Said::about([("network", "ok".to_string())]);

    it.round.read(0, &mut said);

    let mut kinds = kinds(&it.round);
    kinds.sort();
    assert_eq!(
        kinds,
        ["agent.baseline.ready", "port.listen.new"],
        "the agent saying it has a baseline does not silence the rule that judges one"
    );
}
