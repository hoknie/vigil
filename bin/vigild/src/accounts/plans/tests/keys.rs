use serde_json::json;
use vigil_model::AccountChange;
use vigil_users::fixture::users;

use super::harness::{KEY, OTHER, disk, fingerprint, planned};
use crate::accounts::plans::plan;
use crate::accounts::step::Step;

#[test]
fn a_first_key_for_an_account_makes_its_ssh_directory_owned_by_it_and_a_private_file() {
    let steps = planned(
        AccountChange::CreateSshUser {
            user: "www-data".into(),
            line: OTHER.into(),
        },
        &users(),
        &[],
    );

    assert_eq!(
        steps,
        vec![
            Step::Directory {
                path: "/var/www/.ssh".into(),
                uid: 33,
                gid: 33,
                mode: 0o700
            },
            Step::Write {
                path: "/var/www/.ssh/authorized_keys".into(),
                text: format!("{OTHER}\n"),
                uid: 33,
                gid: 33,
                mode: 0o600
            }
        ]
    );
}

#[test]
fn another_key_goes_into_the_file_the_reading_found_the_account_s_keys_in() {
    let steps = planned(
        AccountChange::CreateKey {
            user: "deploy".into(),
            line: OTHER.into(),
        },
        &users(),
        &[("/home/deploy/.ssh/authorized_keys", &format!("{KEY}\n"))],
    );

    assert_eq!(
        steps,
        vec![Step::Write {
            path: "/home/deploy/.ssh/authorized_keys".into(),
            text: format!("{KEY}\n{OTHER}\n"),
            uid: 1000,
            gid: 1000,
            mode: 0o600
        }]
    );
}

#[test]
fn a_key_taken_away_rewrites_its_file_and_a_key_already_gone_changes_nothing() {
    let reading = users();
    let path = "/home/deploy/.ssh/authorized_keys";
    let steps = planned(
        AccountChange::DeleteKey {
            user: "deploy".into(),
            fingerprint: fingerprint(KEY),
        },
        &reading,
        &[(path, &format!("{KEY}\n{OTHER}\n"))],
    );
    assert!(matches!(&steps[0], Step::Write { text, .. } if *text == format!("{OTHER}\n")));

    let disk = disk(&[(path, &format!("{OTHER}\n"))]);
    let complaint = plan(
        &AccountChange::DeleteKey {
            user: "deploy".into(),
            fingerprint: fingerprint(KEY),
        },
        &reading,
        &|at, _| Ok(disk.get(at).cloned()),
    )
    .expect_err("stale");
    assert!(complaint.contains("no longer holds"), "{complaint}");
}

#[test]
fn every_key_of_an_account_is_taken_by_removing_the_files_the_reading_found_them_in() {
    assert_eq!(
        planned(
            AccountChange::DeleteSshUser {
                user: "deploy".into()
            },
            &users(),
            &[]
        ),
        vec![Step::Remove {
            path: "/home/deploy/.ssh/authorized_keys".into(),
            holder: Some(1000)
        }]
    );
}

#[test]
fn a_first_key_for_an_account_whose_ssh_directory_points_elsewhere_is_refused_before_anything_is_written()
 {
    let uid = rustix::process::getuid().as_raw();
    if uid == 0 {
        return;
    }
    let gid = rustix::process::getgid().as_raw();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or(0);
    let home = std::env::temp_dir().join(format!("vigild-plan-home-{stamp}"));
    let elsewhere = std::env::temp_dir().join(format!("vigild-plan-elsewhere-{stamp}"));
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    std::fs::write(elsewhere.join("authorized_keys"), "root's key\n").expect("writes");
    std::os::unix::fs::symlink(&elsewhere, home.join(".ssh")).expect("links");
    let mut reading = users();
    reading.items.insert(
        "account|bob".into(),
        json!({"name": "bob", "uid": uid, "gid": gid, "home": home.to_string_lossy()}),
    );

    let complaint = plan(
        &AccountChange::CreateSshUser {
            user: "bob".into(),
            line: OTHER.into(),
        },
        &reading,
        &crate::files::read,
    )
    .expect_err("a link bob made is not followed");

    assert!(
        complaint.contains("symbolic link"),
        "run as uid {uid}; as root the link would be root's own and followed, so the test \
         says nothing there: {complaint}"
    );
    assert_eq!(
        std::fs::read_to_string(elsewhere.join("authorized_keys")).expect("reads"),
        "root's key\n"
    );
}
