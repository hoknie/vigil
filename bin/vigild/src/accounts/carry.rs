use vigil_model::Killing;

use super::files;
use super::owner::runs_as;
use super::step::Step;
use super::tools;
use super::utility::Utility;
use crate::killing::send;

const SUDOERS_MODE: u32 = 0o440;

pub fn carry(steps: &[Step]) -> Result<String, String> {
    let mut said: Vec<String> = Vec::with_capacity(steps.len());
    for step in steps {
        match one(step) {
            Ok(done) => said.push(done),
            Err(refused) if said.is_empty() => return Err(refused),
            Err(refused) => {
                return Err(format!(
                    "{}; and then it stopped: {refused}",
                    said.join("; ")
                ));
            }
        }
    }
    Ok(said.join("; "))
}

fn one(step: &Step) -> Result<String, String> {
    match step {
        Step::Run { utility, arguments } => tools::run(*utility, arguments),
        Step::Directory {
            path,
            uid,
            gid,
            mode,
        } => files::directory(path, *uid, *gid, *mode, Some(*uid)),
        Step::Write {
            path,
            text,
            uid,
            gid,
            mode,
        } => files::write(path, text, *uid, *gid, *mode, Some(*uid)),
        Step::Remove { path, holder } => files::remove(path, *holder),
        Step::Sudoers { path, text } => sudoers(path, text),
        Step::Signal { pid, uid } => send(*pid, Killing::Terminate, &|pid| {
            uid.is_none_or(|uid| runs_as(pid, uid))
        }),
    }
}

fn sudoers(path: &str, text: &str) -> Result<String, String> {
    files::write_checked(path, text, SUDOERS_MODE, &|candidate| {
        tools::run(
            Utility::Visudo,
            &["-c".to_string(), "-f".to_string(), candidate.to_string()],
        )
        .map_err(|refused| {
            format!("visudo did not accept the new {path}, which was left as it was: {refused}")
        })
    })
    .map(|_| format!("{path} checked by visudo and put in place"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_to_do_is_done_and_says_nothing() {
        assert_eq!(carry(&[]), Ok(String::new()));
    }

    #[test]
    fn a_step_that_fails_after_others_worked_says_which_worked_before_it_stopped() {
        let directory = std::env::temp_dir().join(format!("vigild-carry-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("temp dir");
        let gone = directory.join("never-there");
        let blocked = directory.join("not-a-directory");
        std::fs::write(&blocked, "a file").expect("writes");
        let uid = rustix::process::getuid().as_raw();
        let gid = rustix::process::getgid().as_raw();

        let said = carry(&[
            Step::Remove {
                path: gone.to_string_lossy().into_owned(),
                holder: Some(uid),
            },
            Step::Directory {
                path: blocked.to_string_lossy().into_owned(),
                uid,
                gid,
                mode: 0o700,
            },
        ])
        .expect_err("the second step cannot be done");

        assert!(said.contains("already gone"), "{said}");
        assert!(said.contains("stopped"), "{said}");
        assert!(said.contains("not a directory"), "{said}");
    }
}
