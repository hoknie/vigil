use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronJob {
    pub source: String,
    pub user: String,
    pub schedule: String,
    pub command: String,
}

impl CronJob {
    pub fn of(item: &Value) -> Option<CronJob> {
        Some(CronJob {
            source: item["source"].as_str()?.to_string(),
            user: item["user"].as_str()?.to_string(),
            schedule: item["schedule"].as_str()?.to_string(),
            command: item["command"].as_str()?.to_string(),
        })
    }

    pub fn is_a_script_of_its_directory(&self) -> bool {
        self.command
            .strip_prefix(&self.source)
            .is_some_and(|rest| rest.starts_with('/') && !rest[1..].contains(' '))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    #[test]
    fn a_job_read_from_a_crontab_carries_the_four_facts_a_line_of_it_is_made_of() {
        let job = CronJob::of(&fixture::cron_job(
            "/etc/crontab",
            "root",
            "17 * * * *",
            "/usr/bin/backup",
        ))
        .expect("the four fields are there");

        assert_eq!(job.source, "/etc/crontab");
        assert_eq!(job.user, "root");
        assert_eq!(job.schedule, "17 * * * *");
        assert_eq!(job.command, "/usr/bin/backup");
    }

    #[test]
    fn a_row_of_another_kind_is_not_a_cron_job_and_says_so_rather_than_guessing() {
        assert_eq!(
            CronJob::of(&fixture::unit("nginx.service", "/usr/sbin/nginx", "root")),
            None,
            "a unit handed to the crontab editor is a line nobody would find, in a file \
             nobody would open"
        );
    }

    #[test]
    fn a_script_run_by_a_directory_is_told_apart_from_a_line_in_a_file() {
        let script = CronJob::of(&fixture::cron_job(
            "/etc/cron.daily",
            "root",
            "@daily",
            "/etc/cron.daily/logrotate",
        ))
        .expect("read");
        let line = CronJob::of(&fixture::cron_job(
            "/etc/crontab",
            "root",
            "17 * * * *",
            "cd / && run-parts --report /etc/cron.hourly",
        ))
        .expect("read");

        assert!(
            script.is_a_script_of_its_directory(),
            "there is no line to comment out in /etc/cron.daily: the file itself is the job, \
             and an agent that opened the directory as a file would say something unhelpful"
        );
        assert!(!line.is_a_script_of_its_directory());
    }
}
