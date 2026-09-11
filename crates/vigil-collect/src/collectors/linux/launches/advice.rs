use std::path::Path;

use crate::parsers::AUDIT_KEY;

pub(super) fn rule_not_loaded(source: &Path) -> String {
    format!(
        "auditd is running, nothing recent in {} carries the {AUDIT_KEY} tag: either nobody has run a command since it was last written, or the audit rule is not loaded. To load it, put `-a always,exit -F arch=b64 -S execve -F auid>=1000 -F auid!=unset -k {AUDIT_KEY}` in a file under /etc/audit/rules.d/ (repeat the line with arch=b32 on a host that also runs 32-bit programs) and run `augenrules --load`",
        source.display()
    )
}
