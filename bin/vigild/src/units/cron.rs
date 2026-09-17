use vigil_persistence::{CronJob, commented};

const CEILING_BYTES: usize = 1024 * 1024;

const MODE: u32 = 0o644;

pub fn edit(job: &CronJob, out: bool) -> Result<String, String> {
    if job.is_a_script_of_its_directory() {
        return Err(format!(
            "this job is the file {}, which {} runs whole: there is no line to comment out. \
             Move the file out of that directory, or take away the bit that lets it run",
            job.command, job.source
        ));
    }

    let Some(text) = crate::files::read(&job.source, None)? else {
        return Err(format!(
            "{} is not there any more, so the line the console marked is gone too",
            job.source
        ));
    };
    if text.len() > CEILING_BYTES {
        return Err(format!(
            "{} is {} bytes, past the {CEILING_BYTES} this agent rewrites: it is not a \
             crontab any more, and nothing was written",
            job.source,
            text.len()
        ));
    }

    let written = commented(&text, job, out)?;
    crate::files::write(&job.source, &written, 0, 0, MODE, None)?;

    Ok(match out {
        true => format!(
            "{} written: the line is commented out, and cron stops running it",
            job.source
        ),
        false => format!(
            "{} written: the # is off the line, and cron runs it again",
            job.source
        ),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn job(source: &str, command: &str) -> CronJob {
        CronJob {
            source: source.to_string(),
            user: "root".to_string(),
            schedule: "@reboot".to_string(),
            command: command.to_string(),
        }
    }

    fn scratch(named: &str) -> std::path::PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "vigild-cron-{named}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&directory).expect("temp dir");
        directory
    }

    #[test]
    fn the_file_is_written_with_the_line_hidden_and_read_back_with_it_shown_again() {
        let path = scratch("line").join("root");
        fs::write(&path, "@reboot /usr/local/bin/agent\n").expect("writes");
        let job = job(path.to_str().expect("utf-8"), "/usr/local/bin/agent");

        let said = edit(&job, true).expect("written");

        assert!(said.contains("commented out"), "{said}");
        assert_eq!(
            fs::read_to_string(&path).expect("reads"),
            "#@reboot /usr/local/bin/agent\n"
        );
        edit(&job, false).expect("written back");
        assert_eq!(
            fs::read_to_string(&path).expect("reads"),
            "@reboot /usr/local/bin/agent\n",
            "the file a person keeps is handed back exactly as it was, or the console has \
             edited the host in a way nobody can undo from it"
        );
    }

    #[test]
    fn a_job_that_is_a_whole_file_of_a_run_parts_directory_is_refused_before_anything_is_opened() {
        let refused = edit(&job("/etc/cron.daily", "/etc/cron.daily/logrotate"), true)
            .expect_err("there is no line");

        assert!(
            refused.contains("no line to comment out"),
            "opening a directory as a file would say something about a file type, and the \
             reader would learn nothing about what to do instead: {refused}"
        );
    }

    #[test]
    fn a_crontab_that_went_away_between_the_reading_and_the_keystroke_is_said_to_be_gone() {
        let path = scratch("gone").join("root");

        let refused = edit(
            &job(path.to_str().expect("utf-8"), "/usr/local/bin/agent"),
            true,
        )
        .expect_err("nothing is there");

        assert!(refused.contains("not there any more"), "{refused}");
    }

    #[test]
    fn a_line_no_longer_in_the_file_leaves_the_file_exactly_as_it_was() {
        let path = scratch("moved").join("root");
        fs::write(&path, "@reboot /usr/local/bin/other\n").expect("writes");

        let refused = edit(
            &job(path.to_str().expect("utf-8"), "/usr/local/bin/agent"),
            true,
        )
        .expect_err("it is not there");

        assert!(refused.contains("nothing was written"), "{refused}");
        assert_eq!(
            fs::read_to_string(&path).expect("reads"),
            "@reboot /usr/local/bin/other\n",
            "a crontab rewritten because a marked line had moved is somebody else's job \
             stopped"
        );
    }
}
