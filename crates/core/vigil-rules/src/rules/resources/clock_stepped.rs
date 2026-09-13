use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::resource_finding::{ResourceFinding, build};
use super::resource_limits::ResourceLimits;
use super::resource_view::{Family, ResourceView};
use crate::{Rule, RuleContext};

pub const FINDING_KEY: &str = "resource|clock";

pub struct ClockStepped {
    limits: ResourceLimits,
}

impl ClockStepped {
    pub fn watching(limits: ResourceLimits) -> Self {
        ClockStepped { limits }
    }
}

impl Rule for ClockStepped {
    fn name(&self) -> &'static str {
        "clock_stepped"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let was = ResourceView::new(key, before);
        let now = ResourceView::new(key, after);
        if !now.is(Family::Boot) {
            return None;
        }
        if was.boot_id()? != now.boot_id()? {
            return None;
        }

        let (was_at, now_at) = (was.booted_at()?, now.booted_at()?);
        let stepped = now_at.checked_sub(was_at)?;
        if stepped.unsigned_abs() < self.limits.clock_skew_seconds.unsigned_abs() {
            return None;
        }

        Some(build(
            ResourceFinding {
                kind: KnownKind::ResourceClockSkew,
                severity: Severity::Medium,
                rule: self.name(),
                key,
                finding_key: FINDING_KEY.to_string(),
                object: "host",
                title: format!(
                    "The clock on this host stepped {} seconds {} while the same kernel kept running",
                    stepped.abs(),
                    match stepped > 0 {
                        true => "forward",
                        false => "back",
                    }
                ),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![
                    Evidence {
                        kind: "note".into(),
                        value: format!(
                            "measured against the seconds this host has been up: it says it booted at {now_at}, and at the reading before it said {was_at}"
                        ),
                    },
                    Evidence {
                        kind: "note".into(),
                        value: "times recorded on this host no longer line up with times recorded anywhere else, and a correlation across hosts falls apart without saying so".into(),
                    },
                ],
            },
            ctx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    const ONE_BOOT: &str = "1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e31";

    const ANOTHER_BOOT: &str = "7d3b9a10-2e45-4c81-b6f7-9a0c1d2e3f40";

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        ClockStepped::watching(ResourceLimits::default()).apply(change, &mut ctx)
    }

    fn boot_went(before: serde_json::Value, after: serde_json::Value) -> Change {
        Change::Changed {
            key: "boot|current".into(),
            before,
            after,
        }
    }

    #[test]
    fn a_clock_dragged_back_an_hour_under_the_same_kernel_is_reported_and_says_which_way() {
        let finding = apply(&boot_went(
            fixture::boot(ONE_BOOT, 1_757_419_200),
            fixture::boot(ONE_BOOT, 1_757_415_600),
        ))
        .expect("fires");

        assert_eq!(finding.finding_key, "resource|clock");
        assert_eq!(finding.kind.as_str(), "resource.clock_skew");
        assert_eq!(finding.severity, Severity::Medium);
        assert!(
            finding.title.contains("3600 seconds back"),
            "{}",
            finding.title
        );
    }

    #[test]
    fn a_restart_is_never_also_a_clock_that_jumped() {
        assert!(
            apply(&boot_went(
                fixture::boot(ONE_BOOT, 1_757_419_200),
                fixture::boot(ANOTHER_BOOT, 1_757_720_000),
            ))
            .is_none(),
            "a host that booted again booted at a different moment by definition, and calling \
             that a clock that moved would put a second finding on every reboot for ever"
        );
    }

    #[test]
    fn a_second_of_disagreement_between_two_whole_second_clocks_is_not_a_finding() {
        for moved in [-4i64, -1, 0, 1, 4, 299] {
            assert!(
                apply(&boot_went(
                    fixture::boot(ONE_BOOT, 1_757_419_200),
                    fixture::boot(ONE_BOOT, 1_757_419_200 + moved),
                ))
                .is_none(),
                "moved by {moved}"
            );
        }
    }

    #[test]
    fn the_limit_a_host_runs_on_is_the_one_its_file_names_and_not_one_written_into_this_rule() {
        let strict = ClockStepped::watching(ResourceLimits {
            clock_skew_seconds: 30,
            ..ResourceLimits::default()
        });
        let change = boot_went(
            fixture::boot(ONE_BOOT, 1_757_419_200),
            fixture::boot(ONE_BOOT, 1_757_419_260),
        );
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };

        assert!(strict.apply(&change, &mut ctx).is_some());
        assert!(apply(&change).is_none());
    }
}
