use std::collections::BTreeMap;
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::ptr;

const FIRST_BUFFER: usize = 4096;

const LARGEST_BUFFER: usize = 1024 * 1024;

pub fn name_of_user(uid: u32) -> Option<String> {
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
        if answer != 0 || found.is_null() || found != entry.as_mut_ptr() {
            return None;
        }

        let name = unsafe { entry.assume_init_ref() }.pw_name;
        if name.is_null() {
            return None;
        }
        return Some(
            unsafe { CStr::from_ptr(name) }
                .to_string_lossy()
                .into_owned(),
        );
    }
}

pub fn this_account() -> u32 {
    unsafe { libc::geteuid() }
}

pub fn names_of_users(uids: impl IntoIterator<Item = u32>) -> BTreeMap<u32, String> {
    let mut names = BTreeMap::new();
    for uid in uids {
        if names.contains_key(&uid) {
            continue;
        }
        if let Some(name) = name_of_user(uid) {
            names.insert(uid, name);
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_account_is_the_account_the_files_this_process_creates_belong_to() {
        use std::os::unix::fs::MetadataExt;

        let path = std::env::temp_dir().join(format!("vigil-account-{}", std::process::id()));
        std::fs::write(&path, b"").expect("a temporary file");
        let owner = std::fs::metadata(&path).expect("written").uid();
        let _ = std::fs::remove_file(&path);

        assert_eq!(this_account(), owner);
    }

    #[test]
    fn the_superuser_is_named_root() {
        assert_eq!(name_of_user(0).as_deref(), Some("root"));
    }

    #[test]
    fn the_account_running_this_test_is_named_as_the_system_names_it() {
        let uid = unsafe { libc::getuid() };
        let expected = std::env::var("USER").ok();

        let name = name_of_user(uid);

        assert!(name.is_some(), "uid {uid} has a name on every Mac");
        if let Some(expected) = expected.filter(|_| uid != 0) {
            assert_eq!(name.as_deref(), Some(expected.as_str()));
        }
    }

    #[test]
    fn a_number_no_account_holds_has_no_name_rather_than_an_empty_one() {
        assert_eq!(name_of_user(3_999_999_999), None);
    }

    #[test]
    fn each_number_is_asked_once_and_one_with_no_account_is_left_out() {
        let names = names_of_users([0, 0, 3_999_999_999]);

        assert_eq!(names.len(), 1);
        assert_eq!(names.get(&0).map(String::as_str), Some("root"));
    }
}
