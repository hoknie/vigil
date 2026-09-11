use std::io;

pub fn absent(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::NotFound
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refusal_to_look_is_not_a_file_that_is_not_there() {
        assert!(!absent(&io::Error::from(io::ErrorKind::PermissionDenied)));
        assert!(!absent(&io::Error::from(io::ErrorKind::InvalidInput)));
        assert!(absent(&io::Error::from(io::ErrorKind::NotFound)));
    }
}
