use std::collections::BTreeMap;

use serde_json::{Value, json};
use vigil_model::Snapshot;

use crate::parsers::{LaunchReading, launches_snapshot, parse_audit_log};
use vigil_collect::Presence;

const LOG: &str = concat!(
    r#"type=SYSCALL msg=audit(1757419203.412:3421): arch=c000003e syscall=59 success=yes exit=0 ppid=2143 pid=2170 auid=1000 uid=1000 tty=pts0 ses=3 comm="nc" exe="/usr/bin/nc.openbsd" key="vigil_exec""#,
    "\n",
    r#"type=EXECVE msg=audit(1757419203.412:3421): argc=4 a0="nc" a1="-l" a2="-p" a3="4444""#,
    "\n",
    r#"type=SYSCALL msg=audit(1757419204.900:3422): arch=c000003e syscall=59 success=yes exit=0 ppid=1 pid=2200 auid=0 uid=0 tty=(none) ses=4 comm="payload" exe="/dev/shm/payload" key="vigil_exec""#,
    "\n",
    r#"type=EXECVE msg=audit(1757419204.900:3422): argc=1 a0="payload""#,
    "\n",
    r#"type=SYSCALL msg=audit(1757419205.100:3423): arch=c000003e syscall=59 success=yes exit=0 ppid=1 pid=2300 auid=4242 uid=4242 tty=(none) ses=5 comm="mysql" exe="/usr/bin/mysql" key="vigil_exec""#,
    "\n",
    r#"type=EXECVE msg=audit(1757419205.100:3423): argc=2 a0="mysql" a1="-pdatabasesecret""#,
    "\n",
    r#"type=SYSCALL msg=audit(1757419206.700:3424): arch=c000003e syscall=59 success=yes exit=0 ppid=1 pid=2400 auid=0 uid=0 tty=(none) ses=6 comm="id" exe="/usr/bin/id" key="vigil_exec""#,
    "\n",
    r#"type=EXECVE msg=audit(1757419206.700:3424): argc=0"#,
    "\n",
    r#"type=SYSCALL msg=audit(1757419207.000:3425): arch=c000003e syscall=59 success=yes exit=0 ppid=2143 pid=2500 auid=1000 uid=1000 tty=pts0 ses=3 comm="nc" exe="/usr/bin/nc.openbsd" key="vigil_exec""#,
    "\n",
    r#"type=EXECVE msg=audit(1757419207.000:3425): argc=2 a0="nc" a1="-z""#,
    "\n",
);

fn where_it_landed(executable: &str) -> Presence {
    match executable {
        "/dev/shm/payload" => Presence::Gone,
        "/usr/bin/mysql" => Presence::NotShown,
        _ => Presence::OnDisk,
    }
}

fn already_capped() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "launches|capped".to_string(),
        json!({"named": false, "reason": "an earlier reading reached the limit"}),
    )])
}

pub fn launches() -> Snapshot {
    let reading = parse_audit_log(LOG.as_bytes(), true);
    let logins = BTreeMap::from([(1000, "alice".to_string()), (0, "root".to_string())]);

    launches_snapshot(
        "2026-09-09T09:00:00.000Z",
        &already_capped(),
        &LaunchReading {
            executions: &reading.executions,
            logins: &logins,
            any_unnamed: true,
            keep_arguments: true,
            on_disk: &where_it_landed,
            from_plugin: true,
            dropped: true,
        },
    )
}
