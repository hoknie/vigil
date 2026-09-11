use vigil_model::Finding;

use crate::budget::{CEILING_PERCENT, CPU, Crossing, RESIDENT, resident};
use crate::helpers::{agent_finding, rfc3339};

use super::Round;

const KILOBYTES_IN_A_MEGABYTE: u64 = 1_024;
const WHY_IT_CLOSED: &str = "three checks in a row under the ceiling, so the finding closes on a cost that has stayed down rather than on one reading";

impl Round {
    pub(super) fn take_budget(&mut self) {
        let resident_kb = resident::kilobytes();
        let duty_percent = self.meter.duty_percent();
        self.shared
            .with(|state| state.record_budget(duty_percent, resident_kb));

        let mut findings = Vec::new();
        if let Some(crossing) = self.meter.check_cpu() {
            findings.push(self.of_cpu(crossing, duty_percent.unwrap_or_default()));
        }
        if let Some(crossing) = self.meter.check_resident(resident_kb) {
            findings.push(self.of_resident(crossing, resident_kb.unwrap_or_default()));
        }
        if findings.is_empty() {
            return;
        }

        let fresh = self.remember(&findings);
        for finding in &fresh {
            eprintln!(
                "{} [{}] {} — {}",
                rfc3339::now(),
                finding.severity,
                finding.kind,
                finding.title
            );
        }
        self.shared.with(|state| state.record_findings(&fresh));
        self.delivery.send(&fresh);
    }

    fn of_cpu(&self, crossing: Crossing, duty_percent: f64) -> Finding {
        match crossing {
            Crossing::Exceeded => agent_finding::budget_exceeded(
                CPU,
                &format!(
                    "This agent costs {duty_percent:.2} % of one core, over the {CEILING_PERCENT} % it is allowed"
                ),
                &self.meter,
            ),
            Crossing::Recovered => agent_finding::budget_recovered(
                CPU,
                &format!("This agent is back inside its share of one core, at {duty_percent:.2} %"),
                WHY_IT_CLOSED,
            ),
        }
    }

    fn of_resident(&self, crossing: Crossing, resident_kb: u64) -> Finding {
        let held = resident_kb / KILOBYTES_IN_A_MEGABYTE;
        let ceiling = resident::CEILING_KB / KILOBYTES_IN_A_MEGABYTE;

        match crossing {
            Crossing::Exceeded => agent_finding::budget_exceeded(
                RESIDENT,
                &format!(
                    "This agent holds {held} MB of memory, over the {ceiling} MB it is allowed"
                ),
                &self.meter,
            ),
            Crossing::Recovered => agent_finding::budget_recovered(
                RESIDENT,
                &format!(
                    "This agent holds {held} MB of memory again, under the {ceiling} MB it is allowed"
                ),
                WHY_IT_CLOSED,
            ),
        }
    }
}
