use crate::parsers::{AUDIT_KEY, Launched, REDACTED_AT_THE_SOURCE};
use vigil_collect::redact;

pub const AUID_UNSET: u32 = u32::MAX;

pub const ARGUMENTS_WRITTEN: usize = 16 * 1024;

pub fn audit_records(launched: &Launched, keep_arguments: bool) -> String {
    let id = format!(
        "{}.{:03}:{}",
        launched.seconds, launched.milliseconds, launched.sequence
    );

    let mut execve = format!(
        "type=EXECVE msg=audit({id}): argc={}",
        launched.arguments.len()
    );
    if keep_arguments && !launched.arguments.is_empty() {
        let clean = redact(&launched.arguments);
        execve.push_str(&format!(
            " a0={} {REDACTED_AT_THE_SOURCE}={}",
            encoded(&cut(&clean.text)),
            match clean.redacted {
                true => "yes",
                false => "no",
            }
        ));
    }

    format!(
        "type=SYSCALL msg=audit({id}): arch=eslogger syscall=execve success=yes pid={} ppid={} \
         auid={} uid={} euid={} exe={} key=\"{AUDIT_KEY}\"\n{execve}\n",
        launched.pid,
        launched.ppid,
        launched.auid,
        launched.uid,
        launched.euid,
        encoded(&launched.executable),
    )
}

pub fn somebody_launched(launched: &Launched) -> bool {
    launched.auid != AUID_UNSET
}

fn cut(text: &str) -> String {
    match text.len() > ARGUMENTS_WRITTEN {
        false => text.to_string(),
        true => {
            let mut end = ARGUMENTS_WRITTEN;
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            text[..end].to_string()
        }
    }
}

fn encoded(value: &str) -> String {
    let plain = !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'"' && byte != b'\'');
    match plain {
        true => format!("\"{value}\""),
        false => value.bytes().map(|byte| format!("{byte:02X}")).collect(),
    }
}
