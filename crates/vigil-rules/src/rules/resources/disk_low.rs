use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::resource_finding::{ResourceFinding, build, room};
use super::resource_limits::ResourceLimits;
use super::resource_view::{Family, ResourceView};
use crate::{Rule, RuleContext};

pub struct DiskLow {
    limits: ResourceLimits,
}

impl DiskLow {
    pub fn watching(limits: ResourceLimits) -> Self {
        DiskLow { limits }
    }
}

pub fn finding_key(mount: &str) -> String {
    format!("resource|disk|{mount}")
}

impl Rule for DiskLow {
    fn name(&self) -> &'static str {
        "disk_low"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Added { key, after } => (key, None, after),
            Change::Removed { .. } => return None,
        };

        let now = ResourceView::new(key, after);
        if !now.is(Family::Filesystem) || now.read_only() {
            return None;
        }
        if !self.limits.below_the_disk_limit(now.free_percent_step()) {
            return None;
        }
        if let Some(before) = before {
            let was = ResourceView::new(key, before);
            if self.limits.below_the_disk_limit(was.free_percent_step()) {
                return None;
            }
        }

        Some(build(
            ResourceFinding {
                kind: KnownKind::ResourceDiskLow,
                severity: Severity::High,
                rule: self.name(),
                key,
                finding_key: finding_key(now.mount()),
                object: "filesystem",
                title: format!(
                    "Less than {}% of {} is free, and a host with no room on it stops serving and stops being watched",
                    self.limits.disk_free_percent,
                    now.mount()
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![
                    room(&now),
                    Evidence {
                        kind: "note".into(),
                        value: "free space is read in steps of five percent: what this says is that the step is under the limit, not that the limit was crossed to the byte".into(),
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
    use crate::rules::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        DiskLow::watching(ResourceLimits::default()).apply(change, &mut ctx)
    }

    fn filled(before: u64, after: u64) -> Change {
        Change::Changed {
            key: "fs|/var".into(),
            before: fixture::filesystem("/var", Some(before), Some(85)),
            after: fixture::filesystem("/var", Some(after), Some(85)),
        }
    }

    #[test]
    fn a_filesystem_that_fell_under_the_limit_is_reported_under_a_key_naming_the_mount_point() {
        let finding = apply(&filled(15, 5)).expect("fires");

        assert_eq!(finding.finding_key, "resource|disk|/var");
        assert_eq!(finding.kind.as_str(), "resource.disk_low");
        assert_eq!(finding.severity, Severity::High);
        assert!(finding.title.contains("/var"), "{}", finding.title);
    }

    #[test]
    fn a_filesystem_already_under_the_limit_that_filled_further_is_not_a_second_finding() {
        assert!(
            apply(&filled(5, 0)).is_none(),
            "the store raises the counter on a finding it has already seen; a rule that fires \
             again on every step down turns one filling disk into a stream of findings"
        );
    }

    #[test]
    fn a_filesystem_that_was_emptied_says_nothing_here() {
        assert!(apply(&filled(5, 40)).is_none());
        assert!(apply(&filled(40, 35)).is_none());
    }

    #[test]
    fn a_filesystem_mounted_when_it_was_already_full_is_reported_the_first_time_it_is_seen() {
        let change = Change::Added {
            key: "fs|/srv".into(),
            after: fixture::filesystem("/srv", Some(0), Some(85)),
        };

        assert_eq!(
            apply(&change).expect("fires").finding_key,
            "resource|disk|/srv"
        );
    }

    #[test]
    fn a_filesystem_nothing_can_write_to_is_never_one_that_is_running_out_of_room() {
        let mut full = fixture::filesystem("/boot/efi", Some(0), Some(85));
        full["read_only"] = serde_json::json!(true);
        let change = Change::Added {
            key: "fs|/boot/efi".into(),
            after: full,
        };

        assert!(
            apply(&change).is_none(),
            "a read-only filesystem is as full as it was built to be, and nothing on this host \
             is about to fail because of it"
        );
    }

    #[test]
    fn a_filesystem_that_would_not_say_how_full_it_is_is_never_one_that_is_full() {
        let change = Change::Added {
            key: "fs|/var".into(),
            after: fixture::filesystem("/var", None, Some(85)),
        };

        assert!(apply(&change).is_none());
    }
}
