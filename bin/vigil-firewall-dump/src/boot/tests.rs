use std::fs;
#[cfg(target_os = "macos")]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::ExitCode;

#[cfg(target_os = "macos")]
use vigil_firewall::{DUMP_FILE, FirewallDump};

use super::run::run;
use crate::cli::Options;

fn workspace(named: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "vigil-firewall-dump-{}-{named}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&at);
    at
}

#[cfg(target_os = "macos")]
#[test]
fn on_this_mac_every_question_is_answered_or_says_why_and_the_file_is_the_owners_alone() {
    let at = workspace("this-mac");
    let options = Options {
        directory: at.display().to_string(),
        ..Options::default()
    };

    assert_eq!(run(&options), ExitCode::SUCCESS);

    let path = at.join(DUMP_FILE);
    let written: FirewallDump =
        serde_json::from_str(&fs::read_to_string(&path).expect("the dump")).expect("parses");
    assert!(written.asked.contains_key("application firewall"));
    assert!(written.asked.contains_key("pf info"));
    for (key, answer) in &written.asked {
        assert!(
            answer.answered() || answer.why.is_some() || answer.status.is_some(),
            "{key}: an answer that failed says how"
        );
    }
    assert!(
        written.asked["application firewall"].answered(),
        "socketfilterfw answers any account"
    );
    assert_eq!(
        fs::metadata(&path).expect("stat").permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::read_dir(&at).expect("listed").count(),
        1,
        "no scratch file and no half-written temporary is left behind"
    );
    let _ = fs::remove_dir_all(&at);
}

#[cfg(target_os = "macos")]
#[test]
fn what_this_program_writes_on_a_mac_is_what_the_collector_of_the_agent_reads() {
    use vigil_collect::Collector;
    use vigil_firewall::FirewallCollector;

    let at = workspace("read-back");
    let options = Options {
        directory: at.display().to_string(),
        ..Options::default()
    };
    run(&options);

    let read = FirewallCollector::with_path(
        || "2026-09-19T12:00:00.000Z".to_string(),
        at.join(DUMP_FILE),
    )
    .collect()
    .expect("the file this program wrote is read by the agent");

    assert!(read.items.contains_key("fw-application|socketfilterfw"));
    let _ = fs::remove_dir_all(&at);
}

#[cfg(not(target_os = "macos"))]
#[test]
fn off_a_mac_it_says_the_firewall_is_read_another_way_and_writes_nothing() {
    let at = workspace("elsewhere");
    let options = Options {
        directory: at.display().to_string(),
        ..Options::default()
    };

    assert_eq!(run(&options), ExitCode::from(2));
    assert!(!at.exists());
}
