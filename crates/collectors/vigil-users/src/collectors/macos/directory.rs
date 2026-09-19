use std::ffi::CStr;
use std::sync::Mutex;

use vigil_collect::PasswdEntry;

use crate::parsers::{DirectoryAccount, GroupEntry};

pub(super) const MOST_ENTRIES: usize = 65_536;

static ENUMERATING: Mutex<()> = Mutex::new(());

pub(super) fn accounts() -> Vec<DirectoryAccount> {
    let _alone = ENUMERATING
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut found = Vec::new();

    unsafe { libc::setpwent() };
    while found.len() < MOST_ENTRIES {
        let entry = unsafe { libc::getpwent() };
        if entry.is_null() {
            break;
        }
        if let Some(account) = account_of(unsafe { &*entry }) {
            found.push(account);
        }
    }
    unsafe { libc::endpwent() };

    found
}

pub(super) fn groups() -> Vec<GroupEntry> {
    let _alone = ENUMERATING
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut found = Vec::new();

    unsafe { libc::setgrent() };
    while found.len() < MOST_ENTRIES {
        let entry = unsafe { libc::getgrent() };
        if entry.is_null() {
            break;
        }
        if let Some(group) = group_of(unsafe { &*entry }) {
            found.push(group);
        }
    }
    unsafe { libc::endgrent() };

    found
}

fn account_of(entry: &libc::passwd) -> Option<DirectoryAccount> {
    Some(DirectoryAccount {
        entry: PasswdEntry {
            name: text(entry.pw_name)?,
            uid: entry.pw_uid,
            gid: entry.pw_gid,
            home: text(entry.pw_dir).unwrap_or_default(),
            shell: text(entry.pw_shell).unwrap_or_default(),
        },
        password: text(entry.pw_passwd).unwrap_or_default(),
    })
}

fn group_of(entry: &libc::group) -> Option<GroupEntry> {
    let mut members = Vec::new();
    let mut at = entry.gr_mem;
    while !at.is_null() && members.len() < MOST_ENTRIES {
        let member = unsafe { at.read_unaligned() };
        if member.is_null() {
            break;
        }
        members.extend(text(member));
        at = unsafe { at.add(1) };
    }

    Some(GroupEntry {
        name: text(entry.gr_name)?,
        gid: entry.gr_gid,
        members,
    })
}

fn text(field: *const libc::c_char) -> Option<String> {
    if field.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(field) }
            .to_string_lossy()
            .into_owned(),
    )
}
