use std::path::Path;

use crate::parsers::AUDIT_KEY;

pub(super) const SHIPPED_RULE: &str =
    "-a always,exit -F arch=b64 -S execve -F auid!=unset -k vigil_exec";

pub(super) fn nothing_carries_our_tag(source: &Path) -> String {
    format!(
        "auditd is running and nothing this agent has read carries the {AUDIT_KEY} tag. Two hosts look exactly like this and the records alone do not tell them apart: one where the rule is loaded and nobody has run a command in a login session yet, and one where the rule was never loaded. `auditctl -l | grep {AUDIT_KEY}` answers which. If it prints nothing, put `{SHIPPED_RULE}` in a file under /etc/audit/rules.d/ (repeat the line with arch=b32 on a host that also runs 32-bit programs) and run `augenrules --load`; if it prints the rule, there is nothing to fix here and this agent will say so as soon as one record arrives. What a service, a timer or a container runs carries no loginuid and is not this collector's subject either way; {} is what was read",
        source.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_this_text_hands_an_operator_is_the_rule_this_product_ships() {
        let shipped = include_str!("../../../../../../packaging/audit/vigil-exec.rules");

        assert!(
            shipped.lines().any(|line| line.trim() == SHIPPED_RULE),
            "the text tells an operator to paste a line; a line that is not the one in \
             packaging/audit/vigil-exec.rules is a host configured differently from every \
             host this product installed itself on:\n{shipped}"
        );
    }

    #[test]
    fn the_threshold_that_lost_root_is_in_neither_the_rule_nor_the_text() {
        let shipped = include_str!("../../../../../../packaging/audit/vigil-exec.rules");

        for line in shipped.lines().filter(|line| !line.starts_with('#')) {
            assert!(
                !line.contains("auid>="),
                "an interactive root session has loginuid 0, so a threshold of a thousand \
                 makes this rule match nothing a person does on a host administered as root: \
                 {line}"
            );
        }
        assert!(!SHIPPED_RULE.contains("auid>="));
        assert!(SHIPPED_RULE.contains("auid!=unset"));
    }

    #[test]
    fn the_text_names_both_hosts_that_look_alike_and_the_command_that_tells_them_apart() {
        let said = nothing_carries_our_tag(Path::new("/var/log/audit/audit.log"));

        assert!(said.contains("auditctl -l"), "{said}");
        assert!(said.contains("augenrules --load"), "{said}");
        assert!(
            said.contains("nobody has run a command") && said.contains("never loaded"),
            "a text that names one of the two states this agent cannot tell apart sends an \
             operator to fix a rule that may already be loaded: {said}"
        );
    }
}
