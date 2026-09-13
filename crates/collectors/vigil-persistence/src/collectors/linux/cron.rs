use std::path::Path;

use crate::parsers::{CronEntry, CronFormat, cron_script, parse_crontab};

use super::files::{read_capped, sorted_files};
use super::{CRON_DIRECTORY, CRON_SCRIPT_DIRECTORIES, CRON_SPOOLS, CRONTAB};

pub(super) fn read_cron() -> Vec<CronEntry> {
    let mut jobs = Vec::new();

    if let Some(text) = read_capped(Path::new(CRONTAB)) {
        jobs.extend(parse_crontab(&text, CRONTAB, CronFormat::WithUser, "root"));
    }

    for path in sorted_files(Path::new(CRON_DIRECTORY)) {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.contains('.') || name.ends_with('~') {
            continue;
        }
        if let Some(text) = read_capped(&path) {
            let source = path.to_string_lossy().into_owned();
            jobs.extend(parse_crontab(&text, &source, CronFormat::WithUser, "root"));
        }
    }

    for spool in CRON_SPOOLS {
        for path in sorted_files(Path::new(spool)) {
            let Some(user) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if let Some(text) = read_capped(&path) {
                let source = path.to_string_lossy().into_owned();
                jobs.extend(parse_crontab(&text, &source, CronFormat::ForOneUser, user));
            }
        }
    }

    for (directory, schedule) in CRON_SCRIPT_DIRECTORIES {
        for path in sorted_files(Path::new(directory)) {
            jobs.push(cron_script(directory, &path.to_string_lossy(), schedule));
        }
    }

    jobs
}
