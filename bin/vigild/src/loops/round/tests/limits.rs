use vigil_collect::Health;
use vigil_model::{CollectorState, Finding};
use vigil_store::Store;

use super::harness::{Watching, raised, shown, snapshot, watching};
use crate::types::Said;

const OTHERS: &str = "the sockets held by the processes of every other account are not seen";

fn of_kind(findings: &[Finding], kind: &str) -> usize {
    findings
        .iter()
        .filter(|finding| finding.kind.as_str() == kind)
        .count()
}

fn limited(it: &Watching, said: Option<&str>) {
    *it.collector.limit.lock().expect("not poisoned") = said.map(str::to_string);
}

fn healthy(it: &Watching, health: Health) {
    *it.collector.health.lock().expect("not poisoned") = health;
}

fn open(it: &Watching, key: &str, kind: &str) -> bool {
    it.round
        .store
        .open_finding(key, kind)
        .expect("readable")
        .is_some()
}

#[test]
fn a_limit_that_did_not_move_is_said_once_whatever_the_number_of_checks() {
    let mut it = watching("limit-once", Health::Ok, Vec::new());
    limited(&it, Some(OTHERS));
    let mut said = Said::about([("network", "ok".to_string())]);

    for _ in 0..10 {
        it.round.take_health(true, &mut said);
    }

    let findings = raised(&it.round);
    assert_eq!(of_kind(&findings, "agent.collector.limited"), 1);
    assert_eq!(of_kind(&findings, "agent.collector.degraded"), 0);
    assert!(open(
        &it,
        "agent.collector.limited|network",
        "agent.collector.limited"
    ));
    assert_eq!(shown(&it.round), CollectorState::Limited);
}

#[test]
fn a_failure_on_top_of_a_limit_opens_and_closes_apart_from_it() {
    let mut it = watching("limit-and-failure", Health::Ok, Vec::new());
    limited(&it, Some(OTHERS));
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.take_health(true, &mut said);

    healthy(
        &it,
        Health::Degraded("the source of logins stopped answering".into()),
    );
    it.round.take_health(true, &mut said);

    assert!(open(
        &it,
        "agent.collector|network",
        "agent.collector.degraded"
    ));
    assert!(open(
        &it,
        "agent.collector.limited|network",
        "agent.collector.limited"
    ));
    assert_eq!(shown(&it.round), CollectorState::Degraded);

    healthy(&it, Health::Ok);
    it.round.take_health(true, &mut said);

    let findings = raised(&it.round);
    assert_eq!(of_kind(&findings, "agent.collector.recovered"), 1);
    assert!(!open(
        &it,
        "agent.collector|network",
        "agent.collector.degraded"
    ));
    assert!(
        open(
            &it,
            "agent.collector.limited|network",
            "agent.collector.limited"
        ),
        "the failure went and the limit of the mode did not"
    );
    assert_eq!(of_kind(&findings, "agent.collector.limit_lifted"), 0);
    assert_eq!(shown(&it.round), CollectorState::Limited);
}

#[test]
fn a_limit_that_went_away_is_closed_by_its_own_kind() {
    let mut it = watching("limit-lifted", Health::Ok, Vec::new());
    limited(&it, Some(OTHERS));
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.take_health(true, &mut said);

    limited(&it, None);
    it.round.take_health(true, &mut said);

    let findings = raised(&it.round);
    let lifted = findings
        .iter()
        .find(|finding| finding.kind.as_str() == "agent.collector.limit_lifted")
        .expect("the end of a limit is said");
    assert_eq!(lifted.finding_key, "agent.collector.limited|network");
    assert!(
        lifted
            .evidence
            .iter()
            .any(|evidence| evidence.kind == "was" && evidence.value == OTHERS)
    );
    assert!(!open(
        &it,
        "agent.collector.limited|network",
        "agent.collector.limited"
    ));
    assert_eq!(of_kind(&findings, "agent.collector.recovered"), 0);
    assert_eq!(shown(&it.round), CollectorState::Ok);
}

#[test]
fn a_limit_left_by_the_previous_run_is_closed_after_the_first_reading_when_it_is_gone() {
    let mut it = watching("limit-before", Health::Ok, vec![snapshot(&[22])]);
    let mut said = Said::about([("network", "ok".to_string())]);
    said.limited_before("network", OTHERS.to_string());

    it.round.read(0, &mut said);

    assert_eq!(
        of_kind(&raised(&it.round), "agent.collector.limit_lifted"),
        1,
        "an agent started again as root closes what the agent of an account left open"
    );
    assert!(!said.limit_unconfirmed("network"));
}

#[test]
fn a_limit_left_by_the_previous_run_that_still_stands_is_not_said_again() {
    let mut it = watching("limit-stands", Health::Ok, vec![snapshot(&[22])]);
    limited(&it, Some(OTHERS));
    let mut said = Said::about([("network", "ok".to_string())]);
    said.limited_before("network", OTHERS.to_string());

    it.round.read(0, &mut said);

    let findings = raised(&it.round);
    assert_eq!(of_kind(&findings, "agent.collector.limited"), 0);
    assert_eq!(of_kind(&findings, "agent.collector.limit_lifted"), 0);
    assert!(!said.limit_unconfirmed("network"));
}

#[test]
fn a_degraded_left_by_the_previous_version_that_is_now_a_limit_is_recovered_and_the_limit_opened() {
    let mut it = watching(
        "from-the-previous-version",
        Health::Ok,
        vec![snapshot(&[22])],
    );
    limited(&it, Some(OTHERS));
    let mut said = Said::about([("network", "ok".to_string())]);
    said.opened("network", format!("degraded — {OTHERS}; run as root"));

    it.round.take_health(true, &mut said);

    let findings = raised(&it.round);
    assert_eq!(
        of_kind(&findings, "agent.collector.recovered"),
        1,
        "what the previous version called a failure was none, and the history says so once"
    );
    assert_eq!(of_kind(&findings, "agent.collector.limited"), 1);
    assert_eq!(shown(&it.round), CollectorState::Limited);
}
