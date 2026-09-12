use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::resource_finding::{ResourceFinding, build};
use super::resource_view::{Family, ResourceView};
use crate::{Rule, RuleContext};

pub const FINDING_KEY: &str = "resource|boot";

pub struct HostRebooted;

impl Rule for HostRebooted {
    fn name(&self) -> &'static str {
        "host_rebooted"
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

        let (was_boot, now_boot) = (was.boot_id()?, now.boot_id()?);
        if was_boot == now_boot {
            return None;
        }

        Some(build(
            ResourceFinding {
                kind: KnownKind::ResourceReboot,
                severity: Severity::Low,
                rule: self.name(),
                key,
                finding_key: FINDING_KEY.to_string(),
                object: "host",
                title: "This host restarted: the kernel running on it is not the one that was running at the last reading".to_string(),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![
                    Evidence {
                        kind: "note".into(),
                        value: format!("the boot this host is running is {now_boot}, and it was {was_boot}"),
                    },
                    lasted(was.booted_at(), now.booted_at()),
                ],
            },
            ctx,
        ))
    }
}

fn lasted(was: Option<i64>, now: Option<i64>) -> Evidence {
    let value = match (was, now) {
        (Some(was), Some(now)) if now > was => format!(
            "the boot before this one lasted about {} seconds, and this host started again at {now}",
            now - was
        ),
        _ => "how long the boot before this one lasted is not known: the moment this host booted \
              was not read on both readings"
            .to_string(),
    };

    Evidence {
        kind: "note".into(),
        value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::fixture;

    const ONE_BOOT: &str = "1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e31";

    const ANOTHER_BOOT: &str = "7d3b9a10-2e45-4c81-b6f7-9a0c1d2e3f40";

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        HostRebooted.apply(change, &mut ctx)
    }

    fn boot_went(before: serde_json::Value, after: serde_json::Value) -> Change {
        Change::Changed {
            key: "boot|current".into(),
            before,
            after,
        }
    }

    #[test]
    fn a_kernel_that_is_not_the_one_that_was_running_is_a_host_that_restarted() {
        let finding = apply(&boot_went(
            fixture::boot(ONE_BOOT, 1_757_419_200),
            fixture::boot(ANOTHER_BOOT, 1_757_720_000),
        ))
        .expect("fires");

        assert_eq!(finding.finding_key, "resource|boot");
        assert_eq!(finding.kind.as_str(), "resource.reboot");
        assert_eq!(finding.severity, Severity::Low);
        assert!(
            finding.evidence[1].value.contains("300800"),
            "{}",
            finding.evidence[1].value
        );
    }

    #[test]
    fn a_clock_that_moved_under_the_same_kernel_is_not_a_restart() {
        assert!(
            apply(&boot_went(
                fixture::boot(ONE_BOOT, 1_757_419_200),
                fixture::boot(ONE_BOOT, 1_757_415_600),
            ))
            .is_none(),
            "the boot identifier is what a restart changes; the moment this host booted moves \
             whenever anything sets the clock"
        );
    }

    #[test]
    fn a_first_reading_says_nothing_because_it_has_nothing_to_compare_with() {
        let change = Change::Added {
            key: "boot|current".into(),
            after: fixture::boot(ONE_BOOT, 1_757_419_200),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_kernel_that_would_not_say_which_boot_this_is_never_reports_a_restart() {
        assert!(
            apply(&boot_went(
                fixture::boot(ONE_BOOT, 1_757_419_200),
                fixture::boot_unreadable(),
            ))
            .is_none(),
            "a host whose boot identifier stopped being readable did not restart, and saying it \
             did on every such reading is the noise this product dies of"
        );
        assert!(
            apply(&boot_went(
                fixture::boot_unreadable(),
                fixture::boot(ONE_BOOT, 1_757_419_200),
            ))
            .is_none()
        );
    }
}
