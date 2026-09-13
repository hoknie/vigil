pub fn basename(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((_, name)) if !name.is_empty() => name,
        _ => path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_that_ends_in_a_slash_is_shown_whole_rather_than_as_nothing() {
        assert_eq!(basename("/usr/bin/nc"), "nc");
        assert_eq!(basename("/usr/bin/"), "/usr/bin/");
        assert_eq!(basename("nc"), "nc");
    }
}
