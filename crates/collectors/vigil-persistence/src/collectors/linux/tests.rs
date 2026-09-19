use std::path::Path;

use super::cron::spools_once;
use super::dialect::CronDialect;
use super::preload::read_preload;
use super::scripts::describe_script;
use super::{
    BOOT_DIRECTORIES, CRON_SCRIPT_DIRECTORIES, CRON_SPOOLS, PRELOAD, PROFILE_DIRECTORIES,
    PersistenceCollector, SYSTEM_PROFILES,
};
use crate::parsers::ScriptFamily;
use vigil_collect::Collector;

#[test]
fn a_shell_profile_under_a_directory_the_unit_keeps_private_is_read_as_not_shown() {
    let script = describe_script(Path::new("/tmp/.hidden/.bashrc"), ScriptFamily::Profile);

    assert_eq!(script.present, None);
    assert!(!script.shown);
    assert_eq!(script.readable, None);
    assert_eq!(script.digest, None);
}

#[test]
fn a_shell_profile_the_agent_can_look_at_answers_whether_it_is_there() {
    let here = describe_script(Path::new("/etc/passwd"), ScriptFamily::Profile);
    let nowhere = describe_script(
        Path::new("/etc/there-is-no-such-file"),
        ScriptFamily::Profile,
    );

    assert_eq!(here.present, Some(true));
    assert!(here.shown);
    assert_eq!(nowhere.present, Some(false));
    assert!(nowhere.shown);
}

#[test]
fn a_file_the_agent_could_not_even_look_at_is_not_a_file_that_is_gone() {
    let too_long = "/".to_string() + &"there-is-no-such-directory/".repeat(600) + "profile";

    let refused = describe_script(Path::new(&too_long), ScriptFamily::Profile);

    assert!(refused.shown, "the path is not one the unit keeps private");
    assert_eq!(
        refused.present, None,
        "'we could not look' and 'it is not there' must not be the same answer"
    );
    assert_eq!(refused.readable, Some(false));
}

#[test]
fn reads_this_host_and_names_itself_in_the_snapshot() {
    let collector = PersistenceCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());

    let snapshot = collector.collect().expect("something is always readable");

    assert_eq!(snapshot.source, "persistence");
    assert_eq!(snapshot.taken_at, "2026-09-09T12:00:00.000Z");
    assert!(
        snapshot.items.contains_key("preload|/etc/ld.so.preload"),
        "the preload file is an item whether or not it exists"
    );
    for key in snapshot.items.keys() {
        assert!(key.contains('|'), "key shape: {key}");
    }
}

#[test]
fn a_preload_file_that_is_not_there_is_recorded_as_not_being_there() {
    let preload = read_preload();

    assert_eq!(preload.path, PRELOAD);
    assert!(preload.present || preload.readable);
}

#[test]
fn a_file_in_cron_d_with_a_dot_in_its_name_is_a_job_where_cronie_runs_it_and_not_where_debian_cron_skips_it()
 {
    assert!(
        CronDialect::Cronie.runs_from_cron_d("backup.job"),
        "cronie on RHEL, Fedora, openSUSE and Arch runs /etc/cron.d/backup.job as root, and \
         an agent that skips it is blind to the one job an intruder names with a dot"
    );
    assert!(!CronDialect::Debian.runs_from_cron_d("backup.job"));
    assert!(CronDialect::Debian.runs_from_cron_d("e2scrub_all"));
    assert!(CronDialect::Cronie.runs_from_cron_d("0hourly"));
}

#[test]
fn what_a_package_manager_or_an_editor_leaves_in_cron_d_is_not_a_job_for_either_cron() {
    for left in [
        "sysstat.rpmnew",
        "raid-check.rpmsave",
        "0hourly.rpmorig",
        "job~",
        ".placeholder",
        "#job#",
    ] {
        assert!(!CronDialect::Cronie.runs_from_cron_d(left), "{left}");
        assert!(!CronDialect::Debian.runs_from_cron_d(left), "{left}");
    }
    assert!(!CronDialect::Debian.runs_from_cron_d("php.dpkg-dist"));
}

#[test]
fn the_crontabs_of_users_are_looked_for_where_debian_rhel_opensuse_and_alpine_keep_them() {
    for spool in [
        "/var/spool/cron/crontabs",
        "/var/spool/cron",
        "/var/spool/cron/tabs",
        "/etc/crontabs",
    ] {
        assert!(CRON_SPOOLS.contains(&spool), "{spool}");
    }
}

#[test]
fn a_spool_that_is_a_link_to_another_is_read_once_under_the_first_name() {
    let root = std::env::temp_dir().join(format!("vigil-spools-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let real = root.join("etc-crontabs");
    let linked = root.join("var-spool-cron-crontabs");
    std::fs::create_dir_all(&real).expect("a directory under the temporary one");
    std::os::unix::fs::symlink(&real, &linked).expect("a link beside it");
    let absent = root.join("not-there");
    let (linked, real, absent) = (
        linked.to_string_lossy().into_owned(),
        real.to_string_lossy().into_owned(),
        absent.to_string_lossy().into_owned(),
    );

    let kept = spools_once(&[linked.as_str(), real.as_str(), absent.as_str()]);
    let _ = std::fs::remove_dir_all(&root);

    assert_eq!(
        kept,
        vec![linked.as_str()],
        "Alpine links /var/spool/cron/crontabs to /etc/crontabs; reading both is every job \
         of root twice under two keys"
    );
}

#[test]
fn a_script_dropped_into_the_periodic_directories_of_alpine_is_a_scheduled_job() {
    for (directory, schedule) in [
        ("/etc/periodic/15min", "*/15 * * * *"),
        ("/etc/periodic/hourly", "@hourly"),
        ("/etc/periodic/daily", "@daily"),
        ("/etc/periodic/weekly", "@weekly"),
        ("/etc/periodic/monthly", "@monthly"),
    ] {
        assert!(
            CRON_SCRIPT_DIRECTORIES.contains(&(directory, schedule)),
            "{directory}: the root crontab of Alpine runs it through run-parts"
        );
    }
}

#[test]
fn the_shell_profiles_a_distribution_ships_under_usr_etc_are_watched_beside_those_in_etc() {
    for profile in [
        "/usr/etc/profile",
        "/usr/etc/bash.bashrc",
        "/usr/etc/environment",
    ] {
        assert!(
            SYSTEM_PROFILES.contains(&profile),
            "{profile}: openSUSE Tumbleweed ships no /etc/profile, and a login shell there runs \
             this one"
        );
    }
    assert!(PROFILE_DIRECTORIES.contains(&"/usr/etc/profile.d"));
    assert!(PROFILE_DIRECTORIES.contains(&"/etc/profile.d"));
}

#[test]
fn what_openrc_runs_from_etc_local_d_at_boot_is_watched_as_a_boot_script() {
    assert!(
        BOOT_DIRECTORIES.contains(&"/etc/local.d"),
        "Alpine has no rc.local: the local service of OpenRC runs every *.start in \
         /etc/local.d as root at boot"
    );
}
