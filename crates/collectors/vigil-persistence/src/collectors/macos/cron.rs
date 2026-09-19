use std::path::Path;

use super::files::{Listed, files_in, listed, read_capped};
use super::places::{CRON_TABLES, CRONTAB, PERIODIC_DIRECTORIES};
use crate::parsers::{CronEntry, CronFormat, cron_script, parse_crontab};

pub(super) fn read_cron() -> (Vec<CronEntry>, Option<String>) {
    let mut jobs = Vec::new();

    if let Some(text) = text_of(Path::new(CRONTAB)) {
        jobs.extend(parse_crontab(&text, CRONTAB, CronFormat::WithUser, "root"));
    }

    let unread = match listed(Path::new(CRON_TABLES)) {
        Listed::Files(tables) => {
            for path in tables {
                let Some(user) = path.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if let Some(text) = text_of(&path) {
                    let source = path.to_string_lossy().into_owned();
                    jobs.extend(parse_crontab(&text, &source, CronFormat::ForOneUser, user));
                }
            }
            None
        }
        Listed::Absent => None,
        Listed::Unreadable(why) => Some(why),
    };

    for (directory, schedule) in PERIODIC_DIRECTORIES {
        for path in files_in(Path::new(directory)) {
            jobs.push(cron_script(directory, &path.to_string_lossy(), schedule));
        }
    }

    (jobs, unread)
}

fn text_of(path: &Path) -> Option<String> {
    read_capped(path)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}
