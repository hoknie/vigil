use std::path::PathBuf;

use vigil_config::Account;

pub fn this() -> Account {
    Account {
        uid: rustix::process::geteuid().as_raw(),
        home: std::env::var_os("HOME").map(PathBuf::from),
    }
}

pub fn named() -> String {
    let uid = this().uid;
    match name_of(uid) {
        Some(name) => format!("{name} (uid {uid})"),
        None => format!("uid {uid}"),
    }
}

pub fn name_of(uid: u32) -> Option<String> {
    let mut buffer = vec![0u8; 4096];
    loop {
        let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
        let mut found: *mut libc::passwd = std::ptr::null_mut();
        let code = unsafe {
            libc::getpwuid_r(
                uid,
                &mut entry,
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut found,
            )
        };
        if code == libc::ERANGE && buffer.len() < 1 << 20 {
            buffer.resize(buffer.len() * 2, 0);
            continue;
        }
        if code != 0 || found.is_null() {
            return None;
        }
        let name = unsafe { std::ffi::CStr::from_ptr(entry.pw_name) };
        return name.to_str().ok().map(str::to_string);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_account_of_this_test_is_read_with_its_effective_uid() {
        assert_eq!(this().uid, rustix::process::geteuid().as_raw());
    }

    #[test]
    fn root_is_named_and_an_unused_uid_is_not() {
        assert_eq!(name_of(0).as_deref(), Some("root"));
        assert_eq!(name_of(4_000_000_000), None);
    }
}
