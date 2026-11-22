use std::io::{self, ErrorKind};

use super::accounts::{name_of_user, this_account};

pub fn denied_to_this_account(error: &io::Error) -> bool {
    denied_to(error.kind(), this_account())
}

pub fn denied_to(kind: ErrorKind, uid: u32) -> bool {
    kind == ErrorKind::PermissionDenied && uid != 0
}

pub fn named_account(uid: u32) -> String {
    match name_of_user(uid) {
        Some(name) => format!("{name} (uid {uid})"),
        None => format!("uid {uid}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refusal_to_an_ordinary_account_is_a_limit() {
        assert!(denied_to(ErrorKind::PermissionDenied, 501));
    }

    #[test]
    fn a_refusal_to_root_is_a_failure() {
        assert!(
            !denied_to(ErrorKind::PermissionDenied, 0),
            "what root is refused is refused by something other than the account, such as the \
             privacy settings of the Mac, and it is fixed rather than accepted"
        );
    }

    #[test]
    fn an_error_other_than_a_refusal_is_a_failure() {
        for kind in [
            ErrorKind::NotFound,
            ErrorKind::InvalidData,
            ErrorKind::Interrupted,
            ErrorKind::Other,
        ] {
            assert!(!denied_to(kind, 501), "{kind:?}");
        }
    }

    #[test]
    fn root_is_named_with_its_uid() {
        assert_eq!(named_account(0), "root (uid 0)");
    }
}
