use crate::{Edit, Entry};

pub(super) const SHIPPED: &str = "\
state_dir: /var/lib/vigil
retention_days: 90

collectors:
  - network

# What this host is expected to do.
suppressions: []

reporters: []
";

pub(super) fn entry(key: &str) -> Entry {
    Entry {
        key: key.into(),
        prefix: false,
        kind: None,
        until: None,
        reason: "the staging api, expected here".into(),
    }
}

pub(super) fn changed(edit: Edit) -> String {
    match edit {
        Edit::Changed { text, .. } => text,
        other => panic!("{other:?}"),
    }
}

pub(super) fn wrote(edit: Edit) -> usize {
    match edit {
        Edit::Changed { entries, .. } => entries,
        other => panic!("{other:?}"),
    }
}
