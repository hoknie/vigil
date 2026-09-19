use std::ffi::CStr;
use std::fs;
use std::mem::MaybeUninit;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::ptr;

use super::Account;

const ROOT: u32 = 0;

const WHEEL: u32 = 0;

const ADMIN: u32 = 80;

const WRITABLE_BY_THE_GROUP: u32 = 0o020;

const WRITABLE_BY_ANYONE: u32 = 0o002;

const FIRST_BUFFER: usize = 4096;

const LARGEST_BUFFER: usize = 1024 * 1024;

pub fn run_as(program: &str) -> Result<Account, String> {
    let me = unsafe { libc::geteuid() };
    if me != ROOT {
        return account(me).ok_or_else(|| format!("uid {me} names no account on this Mac"));
    }

    let real = fs::canonicalize(program).map_err(|error| format!("{program}: {error}"))?;
    let owner = fs::metadata(&real)
        .map_err(|error| format!("{}: {error}", real.display()))?
        .uid();

    let mut at: Option<&Path> = Some(real.as_path());
    while let Some(path) = at {
        let held = fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
        judged(
            &path.display().to_string(),
            held.uid(),
            held.gid(),
            held.mode(),
            owner,
        )?;
        at = path.parent();
    }

    account(owner).ok_or_else(|| {
        format!(
            "{} belongs to uid {owner}, which names no account on this Mac",
            real.display()
        )
    })
}

pub fn judged(path: &str, uid: u32, gid: u32, mode: u32, owner: u32) -> Result<(), String> {
    if uid != ROOT && uid != owner {
        return Err(format!(
            "{path} belongs to uid {uid} and the program to uid {owner}: the first could put \
             a program of its own in place of the second, and it would run as the second"
        ));
    }
    if mode & WRITABLE_BY_ANYONE != 0 {
        return Err(format!(
            "{path} may be written by any account, and any account could put a program of its \
             own in place of the client"
        ));
    }
    if mode & WRITABLE_BY_THE_GROUP != 0 && gid != WHEEL && gid != ADMIN {
        return Err(format!(
            "{path} may be written by the members of group {gid}, and any of them could put a \
             program of its own in place of the client"
        ));
    }
    Ok(())
}

fn account(uid: u32) -> Option<Account> {
    let mut buffer = vec![0 as libc::c_char; FIRST_BUFFER];

    loop {
        let mut entry = MaybeUninit::<libc::passwd>::zeroed();
        let mut found: *mut libc::passwd = ptr::null_mut();
        let answer = unsafe {
            libc::getpwuid_r(
                uid,
                entry.as_mut_ptr(),
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut found,
            )
        };

        if answer == libc::ERANGE && buffer.len() < LARGEST_BUFFER {
            buffer.resize(buffer.len() * 4, 0);
            continue;
        }
        if answer != 0 || found.is_null() {
            return None;
        }

        let entry = unsafe { &*found };
        return Some(Account {
            uid,
            gid: entry.pw_gid,
            name: text(entry.pw_name)?,
            home: text(entry.pw_dir),
        });
    }
}

fn text(pointer: *const libc::c_char) -> Option<String> {
    match pointer.is_null() {
        true => None,
        false => Some(
            unsafe { CStr::from_ptr(pointer) }
                .to_string_lossy()
                .into_owned(),
        ),
    }
}
