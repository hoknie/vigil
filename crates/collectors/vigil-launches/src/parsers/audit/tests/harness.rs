use super::super::log::parse_audit_log;
use super::super::reading::AuditReading;

pub(super) const ONE_LAUNCH: &str = concat!(
    r#"type=SYSCALL msg=audit(1757419203.412:3421): arch=c000003e syscall=59 success=yes exit=0 a0=55d2b5e4a2c0 a1=55d2b5e4a340 a2=55d2b5e4a3a0 a3=8 items=2 ppid=2143 pid=2170 auid=1000 uid=1000 gid=1000 euid=1000 suid=1000 fsuid=1000 egid=1000 sgid=1000 fsgid=1000 tty=pts0 ses=3 comm="nc" exe="/usr/bin/nc.openbsd" subj=unconfined key="vigil_exec""#,
    "\n",
    r#"type=EXECVE msg=audit(1757419203.412:3421): argc=3 a0="nc" a1="-l" a2="4444""#,
    "\n",
    r#"type=CWD msg=audit(1757419203.412:3421): cwd="/home/alice""#,
    "\n",
    r#"type=PATH msg=audit(1757419203.412:3421): item=0 name="/usr/bin/nc" inode=1441806 dev=fd:01 mode=0100755 ouid=0 ogid=0 rdev=00:00 nametype=NORMAL cap_fp=0 cap_fi=0 cap_fe=0 cap_fver=0"#,
    "\n",
    "type=PROCTITLE msg=audit(1757419203.412:3421): proctitle=6E63002D6C0034343434\n",
);

pub(super) fn read(text: &str) -> AuditReading {
    parse_audit_log(text.as_bytes(), true)
}
