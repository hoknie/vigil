use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use vigil_collect::name_of_user;

use super::files::{Listed, listed, read_capped};
use super::places::{HOMES, LAUNCHD_DIRECTORIES, NOT_A_PERSON, PERSONAL_AGENTS};
use crate::parsers::{Domain, LaunchdJob, Scope, launchd_facts, parse_plist};

const PLIST: &str = "plist";

pub(super) struct Home {
    pub(super) owner: String,
    pub(super) path: PathBuf,
}

pub(super) struct Jobs {
    pub(super) jobs: Vec<LaunchdJob>,
    pub(super) unread: Vec<String>,
}

pub(super) fn homes() -> Vec<Home> {
    let Ok(entries) = fs::read_dir(HOMES) else {
        return Vec::new();
    };
    let mut homes: Vec<Home> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || NOT_A_PERSON.contains(&name.as_str()) {
                return None;
            }
            let metadata = fs::metadata(entry.path()).ok()?;
            if !metadata.is_dir() {
                return None;
            }
            Some(Home {
                owner: name_of_user(metadata.uid()).unwrap_or(name),
                path: entry.path(),
            })
        })
        .collect();
    homes.sort_by(|left, right| left.path.cmp(&right.path));
    homes
}

pub(super) fn unseen(homes: &[Home]) -> (usize, Vec<String>, Vec<String>) {
    let mut listed_files = 0usize;
    let mut unlisted = Vec::new();
    let mut unopened = Vec::new();

    let places = LAUNCHD_DIRECTORIES
        .iter()
        .map(|(directory, _, scope)| (PathBuf::from(directory), *scope))
        .chain(
            homes
                .iter()
                .map(|home| (home.path.join(PERSONAL_AGENTS), Scope::Person)),
        );
    for (directory, scope) in places {
        match listed(&directory) {
            Listed::Files(files) => {
                for path in files
                    .iter()
                    .filter(|path| path.extension().is_some_and(|kind| kind == PLIST))
                {
                    listed_files += 1;
                    if scope != Scope::Vendor && fs::File::open(path).is_err() {
                        unopened.push(path.display().to_string());
                    }
                }
            }
            Listed::Absent => {}
            Listed::Unreadable(why) => unlisted.push(why),
        }
    }

    (listed_files, unlisted, unopened)
}

pub(super) fn read_jobs(homes: &[Home]) -> Jobs {
    let mut read = Jobs {
        jobs: Vec::new(),
        unread: Vec::new(),
    };

    for (directory, domain, scope) in LAUNCHD_DIRECTORIES {
        read_directory(Path::new(directory), *domain, *scope, None, &mut read);
    }
    for home in homes {
        read_directory(
            &home.path.join(PERSONAL_AGENTS),
            Domain::Agent,
            Scope::Person,
            Some(&home.owner),
            &mut read,
        );
    }

    read
}

fn read_directory(
    directory: &Path,
    domain: Domain,
    scope: Scope,
    owner: Option<&str>,
    read: &mut Jobs,
) {
    let files = match listed(directory) {
        Listed::Files(files) => files,
        Listed::Absent => return,
        Listed::Unreadable(why) => {
            read.unread.push(why);
            return;
        }
    };

    for path in files
        .into_iter()
        .filter(|path| path.extension().is_some_and(|kind| kind == PLIST))
    {
        read.jobs.push(job(&path, domain, scope, owner));
    }
}

fn job(path: &Path, domain: Domain, scope: Scope, owner: Option<&str>) -> LaunchdJob {
    let (readable, facts) = match read_capped(path) {
        Ok(bytes) => (
            true,
            parse_plist(&bytes)
                .map(|value| launchd_facts(&value))
                .map_err(|refusal| refusal.to_string()),
        ),
        Err(error) => (false, Err(format!("the file cannot be read: {error}"))),
    };

    LaunchdJob {
        path: path.to_string_lossy().into_owned(),
        domain,
        scope,
        owner: owner.map(str::to_string),
        readable,
        facts,
    }
}
