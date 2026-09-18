use std::path::Path;

pub const SHIPPED_DIRECTORY: &str = "/etc/vigil/";

pub fn placed(text: &str, directory: &Path) -> String {
    let here = format!("{}/", directory.display().to_string().trim_end_matches('/'));
    text.split_inclusive('\n')
        .map(|line| match line.trim_start().starts_with('#') {
            true => line.to_string(),
            false => line.replace(SHIPPED_DIRECTORY, &here),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_under_the_shipped_directory_is_moved_to_the_directory_being_written() {
        let text = "collectors_path: /etc/vigil/collectors\nstate_dir: /var/lib/vigil\n";

        assert_eq!(
            placed(text, Path::new("/srv/agent")),
            "collectors_path: /srv/agent/collectors\nstate_dir: /var/lib/vigil\n"
        );
    }

    #[test]
    fn a_comment_is_left_in_the_words_it_was_written_in() {
        let text = "# see /etc/vigil/collectors\nkey: /etc/vigil/x";

        assert_eq!(
            placed(text, Path::new("/srv/agent/")),
            "# see /etc/vigil/collectors\nkey: /srv/agent/x"
        );
    }

    #[test]
    fn the_directory_the_packages_install_to_changes_nothing() {
        let text = "collectors_path: /etc/vigil/collectors\n";

        assert_eq!(placed(text, Path::new("/etc/vigil")), text);
    }
}
