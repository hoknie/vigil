use std::path::Path;

use vigil_config::Installation;

pub const SHIPPED_FOR: Installation = Installation::LINUX;

pub fn placed(text: &str, directory: &Path) -> String {
    placed_on(Installation::here(), text, directory)
}

pub fn placed_on(system: Installation, text: &str, directory: &Path) -> String {
    let moved = system.moved_from(&SHIPPED_FOR, text);
    let shipped = format!("{}/", system.configuration_directory);
    let here = format!("{}/", directory.display().to_string().trim_end_matches('/'));
    moved
        .split_inclusive('\n')
        .map(|line| match line.trim_start().starts_with('#') {
            true => line.to_string(),
            false => line.replace(&shipped, &here),
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
            placed_on(Installation::LINUX, text, Path::new("/srv/agent")),
            "collectors_path: /srv/agent/collectors\nstate_dir: /var/lib/vigil\n"
        );
    }

    #[test]
    fn a_comment_is_left_in_the_words_it_was_written_in() {
        let text = "# see /etc/vigil/collectors\nkey: /etc/vigil/x";

        assert_eq!(
            placed_on(Installation::LINUX, text, Path::new("/srv/agent/")),
            "# see /etc/vigil/collectors\nkey: /srv/agent/x"
        );
    }

    #[test]
    fn the_directory_the_packages_install_to_changes_nothing() {
        let text = "collectors_path: /etc/vigil/collectors\n";

        assert_eq!(
            placed_on(Installation::LINUX, text, Path::new("/etc/vigil")),
            text
        );
    }

    #[test]
    fn on_macos_the_shipped_places_are_the_places_of_macos_before_the_directory_is_placed() {
        let text = "state_dir: /var/lib/vigil\n\
                    socket_path: /run/vigil/vigil.sock\n\
                    collectors_path: /etc/vigil/collectors\n\
                    # see /etc/vigil/collectors\n";

        assert_eq!(
            placed_on(Installation::MACOS, text, Path::new("/usr/local/etc/vigil")),
            "state_dir: /usr/local/var/lib/vigil\n\
             socket_path: /var/run/vigil/vigil.sock\n\
             collectors_path: /usr/local/etc/vigil/collectors\n\
             # see /usr/local/etc/vigil/collectors\n"
        );
        assert_eq!(
            placed_on(Installation::MACOS, text, Path::new("/srv/agent")),
            "state_dir: /usr/local/var/lib/vigil\n\
             socket_path: /var/run/vigil/vigil.sock\n\
             collectors_path: /srv/agent/collectors\n\
             # see /usr/local/etc/vigil/collectors\n",
            "a comment speaks of the place this system installs to, and only a value moves \
             with the directory being written"
        );
    }

    #[test]
    fn this_build_places_what_it_writes_for_the_system_it_was_built_for() {
        let text = "state_dir: /var/lib/vigil\n";

        assert_eq!(
            placed(text, Path::new("/srv/agent")),
            format!("state_dir: {}\n", Installation::here().state_directory)
        );
    }
}
