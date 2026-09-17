use crate::parsers::{CronFormat, parse_crontab};
use crate::types::CronJob;

pub fn commented(text: &str, job: &CronJob, out: bool) -> Result<String, String> {
    let mut written = String::with_capacity(text.len() + 1);
    let mut touched = 0usize;

    for piece in text.split_inclusive('\n') {
        let (body, ending) = split(piece);
        match changed(body, job, out) {
            Some(line) => {
                touched += 1;
                written.push_str(&line);
            }
            None => written.push_str(body),
        }
        written.push_str(ending);
    }

    match (touched, out) {
        (0, true) => Err(format!(
            "no line of {} runs this job: the file changed since the agent read it, and \
             nothing was written",
            job.source
        )),
        (0, false) => Err(format!(
            "no commented-out line of {} is this job: it was put back by hand, or the file \
             changed since the agent read it, and nothing was written",
            job.source
        )),
        _ => Ok(written),
    }
}

fn split(piece: &str) -> (&str, &str) {
    if let Some(body) = piece.strip_suffix("\r\n") {
        return (body, "\r\n");
    }
    match piece.strip_suffix('\n') {
        Some(body) => (body, "\n"),
        None => (piece, ""),
    }
}

fn changed(line: &str, job: &CronJob, out: bool) -> Option<String> {
    match out {
        true => match line.trim_start().starts_with('#') {
            true => None,
            false => is(line, job).then(|| hidden(line)),
        },
        false => {
            let bare = shown(line)?;
            is(&bare, job).then_some(bare)
        }
    }
}

fn hidden(line: &str) -> String {
    let indent = line.len() - line.trim_start().len();
    format!("{}#{}", &line[..indent], &line[indent..])
}

fn shown(line: &str) -> Option<String> {
    let indent = line.len() - line.trim_start().len();
    let rest = line[indent..].strip_prefix('#')?;
    Some(format!("{}{rest}", &line[..indent]))
}

fn is(line: &str, job: &CronJob) -> bool {
    let format = match job.source == "/etc/crontab" || job.source.starts_with("/etc/cron.d/") {
        true => CronFormat::WithUser,
        false => CronFormat::ForOneUser,
    };

    parse_crontab(line, &job.source, format, &job.user)
        .first()
        .is_some_and(|entry| {
            entry.user == job.user && entry.schedule == job.schedule && entry.command == job.command
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYSTEM: &str = "SHELL=/bin/sh\n\
                          # m h dom mon dow user command\n\
                          17 *\t* * *\troot\tcd / && run-parts /etc/cron.hourly\n\
                          @reboot root /usr/local/bin/agent\n";

    fn job(source: &str, user: &str, schedule: &str, command: &str) -> CronJob {
        CronJob {
            source: source.to_string(),
            user: user.to_string(),
            schedule: schedule.to_string(),
            command: command.to_string(),
        }
    }

    fn at_reboot() -> CronJob {
        job("/etc/crontab", "root", "@reboot", "/usr/local/bin/agent")
    }

    #[test]
    fn the_line_of_the_job_gets_a_hash_and_every_other_byte_of_the_file_is_left_alone() {
        let written = commented(SYSTEM, &at_reboot(), true).expect("the line is there");

        assert_eq!(
            written,
            "SHELL=/bin/sh\n\
             # m h dom mon dow user command\n\
             17 *\t* * *\troot\tcd / && run-parts /etc/cron.hourly\n\
             #@reboot root /usr/local/bin/agent\n",
            "the tabs, the settings and the other job are the operator's file, not the \
             agent's to rewrite"
        );
    }

    #[test]
    fn the_hash_comes_off_the_same_line_and_puts_the_file_back_as_it_was() {
        let out = commented(SYSTEM, &at_reboot(), true).expect("commented");

        let back = commented(&out, &at_reboot(), false).expect("uncommented");

        assert_eq!(
            back, SYSTEM,
            "a person who commented a job out at the console and changed their mind gets \
             their file back, byte for byte, or the console has edited the host in a way \
             nobody can undo from it"
        );
    }

    #[test]
    fn a_line_written_by_hand_with_a_space_after_the_hash_is_still_found_and_put_back() {
        let file = "# @reboot root /usr/local/bin/agent\n";

        let back = commented(file, &at_reboot(), false).expect("found");

        assert_eq!(back, " @reboot root /usr/local/bin/agent\n");
    }

    #[test]
    fn a_job_that_is_not_in_the_file_any_more_is_refused_and_the_file_is_not_written() {
        let gone = job("/etc/crontab", "root", "@reboot", "/usr/local/bin/gone");

        let refused = commented(SYSTEM, &gone, true).expect_err("it is not there");

        assert!(refused.contains("nothing was written"), "{refused}");
        assert!(refused.contains("/etc/crontab"), "{refused}");
    }

    #[test]
    fn a_job_already_commented_out_is_not_commented_out_twice() {
        let once = commented(SYSTEM, &at_reboot(), true).expect("commented");

        let twice = commented(&once, &at_reboot(), true).expect_err("there is no live line");

        assert!(
            twice.contains("no line of"),
            "a second hash on the line is a line the console could not put back with the \
             key that took it away: {twice}"
        );
    }

    #[test]
    fn a_user_crontab_line_carries_no_account_and_is_still_the_job_it_is_looked_up_by() {
        let file = "@reboot /tmp/.x/implant\n";
        let implant = job(
            "/var/spool/cron/crontabs/www-data",
            "www-data",
            "@reboot",
            "/tmp/.x/implant",
        );

        assert_eq!(
            commented(file, &implant, true).expect("found"),
            "#@reboot /tmp/.x/implant\n"
        );
    }

    #[test]
    fn a_neighbouring_job_of_the_same_account_is_left_running() {
        let file = "@reboot root /usr/local/bin/agent\n@reboot root /usr/local/bin/other\n";

        let written = commented(file, &at_reboot(), true).expect("found");

        assert_eq!(
            written, "#@reboot root /usr/local/bin/agent\n@reboot root /usr/local/bin/other\n",
            "two jobs of one account differ by their command alone, and an agent that \
             matched on the account would stop the wrong one"
        );
    }

    #[test]
    fn a_file_whose_last_line_has_no_newline_does_not_grow_one() {
        let file = "@reboot root /usr/local/bin/agent";

        assert_eq!(
            commented(file, &at_reboot(), true).expect("found"),
            "#@reboot root /usr/local/bin/agent"
        );
    }

    #[test]
    fn a_file_written_on_another_kind_of_machine_keeps_its_line_endings() {
        let file = "SHELL=/bin/sh\r\n@reboot root /usr/local/bin/agent\r\n";

        assert_eq!(
            commented(file, &at_reboot(), true).expect("found"),
            "SHELL=/bin/sh\r\n#@reboot root /usr/local/bin/agent\r\n",
            "a carriage return the agent swallowed is a diff on every line of a file it was \
             asked to change one line of"
        );
    }

    #[test]
    fn the_indentation_a_person_gave_the_line_stays_in_front_of_the_hash() {
        let file = "  @reboot root /usr/local/bin/agent\n";

        let written = commented(file, &at_reboot(), true).expect("found");

        assert_eq!(written, "  #@reboot root /usr/local/bin/agent\n");
        assert_eq!(
            commented(&written, &at_reboot(), false).expect("back"),
            file
        );
    }
}
