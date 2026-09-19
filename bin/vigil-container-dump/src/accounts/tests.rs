use std::os::unix::fs::MetadataExt;

use super::choosing::judged;
use super::run_as;

const ROOT: u32 = 0;

const SOMEBODY: u32 = 501;

const SOMEBODY_ELSE: u32 = 502;

#[test]
fn a_client_root_owns_in_directories_only_root_may_write_is_run_as_root() {
    assert!(judged("/opt/podman/bin/podman", ROOT, 0, 0o100755, ROOT).is_ok());
    assert!(judged("/opt/podman/bin", ROOT, 0, 0o40755, ROOT).is_ok());
}

#[test]
fn a_client_an_account_owns_is_run_as_that_account_so_replacing_it_gains_nothing() {
    assert!(
        judged(
            "/Applications/Docker.app/Contents/Resources/bin/docker",
            SOMEBODY,
            20,
            0o100755,
            SOMEBODY
        )
        .is_ok(),
        "Docker Desktop is dragged into /Applications by a person and belongs to them; run \
         as root it would hand that person root the next time the job ran"
    );
}

#[test]
fn a_directory_of_another_account_on_the_way_to_the_client_refuses_it() {
    let refusal = judged("/Users/Shared/bin", SOMEBODY_ELSE, 20, 0o40755, SOMEBODY)
        .expect_err("the owner of the directory could replace the client");

    assert!(refusal.contains("/Users/Shared/bin"), "{refusal}");
}

#[test]
fn a_directory_anybody_may_write_to_on_the_way_to_the_client_refuses_it() {
    assert!(judged("/private/tmp", ROOT, 0, 0o41777, ROOT).is_err());
}

#[test]
fn the_groups_that_are_root_already_may_write_on_the_way_and_no_other_group_may() {
    assert!(
        judged("/Applications", ROOT, 80, 0o40775, SOMEBODY).is_ok(),
        "/Applications is writable by admin, and an admin of a Mac may become root anyway"
    );
    assert!(judged("/opt/tools", ROOT, 20, 0o40775, ROOT).is_err());
}

#[test]
fn a_dump_started_by_hand_by_an_account_that_is_not_root_runs_every_client_as_that_account() {
    let me = unsafe { libc::geteuid() };
    if me == ROOT {
        return;
    }

    let account = run_as("/bin/ls").expect("an account");

    assert_eq!(
        account.uid, me,
        "an account that is not root cannot become another one, and running a client as \
         itself hands it nothing it did not have"
    );
}

#[test]
fn a_client_on_this_mac_is_run_as_the_account_that_owns_its_file_when_root_runs_the_dump() {
    let own = std::env::current_exe().expect("this test");
    let owner = std::fs::metadata(&own).expect("stat").uid();
    if unsafe { libc::geteuid() } != ROOT {
        return;
    }

    match run_as(&own.display().to_string()) {
        Ok(account) => assert_eq!(account.uid, owner),
        Err(refusal) => assert!(
            refusal.contains("may be written") || refusal.contains("belongs to uid"),
            "{refusal}"
        ),
    }
}
