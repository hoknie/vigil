use std::fs;
use std::path::{Path, PathBuf};

use super::super::LaunchesCollector;
use crate::spool::cursor_path;

pub(super) const LAUNCH: &str = concat!(
    r#"type=SYSCALL msg=audit(1757419203.412:3421): arch=c000003e syscall=59 success=yes exit=0 items=2 ppid=2143 pid=2170 auid=0 uid=0 tty=pts0 ses=3 comm="id" exe="/usr/bin/id" key="vigil_exec""#,
    "\n",
    r#"type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0="id""#,
    "\n",
);

pub(super) const SECOND_LAUNCH: &str = concat!(
    r#"type=SYSCALL msg=audit(1757419204.000:3422): arch=c000003e syscall=59 success=yes exit=0 auid=0 uid=0 comm="uname" exe="/usr/bin/uname" key="vigil_exec""#,
    "\n",
    r#"type=EXECVE msg=audit(1757419204.000:3422): argc=2 a0="uname" a1="-a""#,
    "\n",
);

pub(super) fn workspace(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("vigil-launches-test");
    fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join(format!("{}-{name}", std::process::id()));
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(spool_beside(&path));
    let _ = fs::remove_file(cursor_path(&spool_beside(&path)));
    let _ = fs::remove_file(plugin_config_beside(&path));
    path
}

pub(super) fn spool_beside(log: &Path) -> PathBuf {
    PathBuf::from(format!("{}.spool", log.display()))
}

pub(super) fn plugin_config_beside(log: &Path) -> PathBuf {
    PathBuf::from(format!("{}.plugin-conf", log.display()))
}

pub(super) fn collector(path: &Path) -> LaunchesCollector {
    LaunchesCollector::with_paths(
        || "2026-09-09T12:00:00.000Z".to_string(),
        false,
        spool_beside(path),
        path,
        plugin_config_beside(path),
    )
}
