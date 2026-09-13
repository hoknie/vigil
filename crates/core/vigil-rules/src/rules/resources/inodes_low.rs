use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::resource_finding::{ResourceFinding, build, room};
use super::resource_limits::ResourceLimits;
use super::resource_view::{Family, ResourceView};
use crate::{Rule, RuleContext};

pub struct InodesLow {
    limits: ResourceLimits,
}

impl InodesLow {
    pub fn watching(limits: ResourceLimits) -> Self {
        InodesLow { limits }
    }
}

pub fn finding_key(mount: &str) -> String {
    format!("resource|inodes|{mount}")
}

impl Rule for InodesLow {
    fn name(&self) -> &'static str {
        "inodes_low"
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
        if !self
            .limits
            .below_the_inode_limit(now.free_inodes_percent_step())
        {
            return None;
        }
        if let Some(before) = before {
            let was = ResourceView::new(key, before);
            if self
                .limits
                .below_the_inode_limit(was.free_inodes_percent_step())
            {
                return None;
            }
        }

        Some(build(
            ResourceFinding {
                kind: KnownKind::ResourceInodeLow,
                severity: Severity::High,
                rule: self.name(),
                key,
                finding_key: finding_key(now.mount()),
                object: "filesystem",
                title: format!(
                    "Less than {}% of the inodes on {} are free, so writes there will fail while the space they need is still there",
                    self.limits.inode_free_percent,
                    now.mount()
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![
                    room(&now),
                    Evidence {
                        kind: "note".into(),
                        value: "a filesystem out of inodes reports room and refuses every new file, which is the failure that reads as a disk that is not full".into(),
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

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        InodesLow::watching(ResourceLimits::default()).apply(change, &mut ctx)
    }

    fn used(before: Option<u64>, after: Option<u64>) -> Change {
        Change::Changed {
            key: "fs|/var".into(),
            before: fixture::filesystem("/var", Some(60), before),
            after: fixture::filesystem("/var", Some(60), after),
        }
    }

    #[test]
    fn a_filesystem_running_out_of_inodes_is_reported_while_it_still_reports_room() {
        let finding = apply(&used(Some(20), Some(5))).expect("fires");

        assert_eq!(finding.finding_key, "resource|inodes|/var");
        assert_eq!(finding.kind.as_str(), "resource.inode_low");
        assert_eq!(finding.severity, Severity::High);
        assert!(
            finding.evidence[0].value.contains("60%"),
            "{}",
            finding.evidence[0].value
        );
    }

    #[test]
    fn a_filesystem_that_counts_no_inodes_is_never_one_that_has_run_out_of_them() {
        assert!(
            apply(&used(Some(20), None)).is_none(),
            "btrfs and xfs report no inode total at all, and reading that as none left would be \
             a finding on every host that runs them"
        );
        assert!(apply(&used(None, None)).is_none());
    }

    #[test]
    fn a_filesystem_already_under_the_limit_is_not_reported_again_on_every_reading() {
        assert!(apply(&used(Some(5), Some(0))).is_none());
    }

    #[test]
    fn inodes_and_free_space_are_two_different_findings_about_one_filesystem() {
        let change = Change::Changed {
            key: "fs|/var".into(),
            before: fixture::filesystem("/var", Some(60), Some(60)),
            after: fixture::filesystem("/var", Some(5), Some(5)),
        };

        let finding = apply(&change).expect("fires");
        assert_eq!(finding.finding_key, "resource|inodes|/var");
        assert_ne!(
            finding.finding_key,
            super::super::disk_low::finding_key("/var"),
            "one filesystem short of both is two histories, and a reader suppressing one of \
             them must not lose the other"
        );
    }
}
