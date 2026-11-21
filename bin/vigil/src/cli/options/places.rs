use std::path::{Path, PathBuf};

use vigil_config::{Installation, Layout};

pub fn socket(asked: Option<&str>, home: Option<&Path>, exists: impl Fn(&Path) -> bool) -> String {
    if let Some(asked) = asked {
        return asked.to_string();
    }
    chosen(
        home.and_then(Layout::socket_under),
        Installation::here().socket,
        exists,
    )
}

pub fn configuration(
    asked: Option<&str>,
    home: Option<&Path>,
    exists: impl Fn(&Path) -> bool,
) -> String {
    if let Some(asked) = asked {
        return asked.to_string();
    }
    chosen(
        home.and_then(Layout::configuration_under),
        Installation::here().configuration,
        exists,
    )
}

pub fn default_socket() -> String {
    socket(None, home().as_deref(), |path| path.exists())
}

pub fn default_configuration() -> String {
    configuration(None, home().as_deref(), |path| path.exists())
}

fn chosen(
    of_the_account: Option<String>,
    of_the_system: &str,
    exists: impl Fn(&Path) -> bool,
) -> String {
    match of_the_account {
        Some(own) if cfg!(target_os = "macos") && exists(Path::new(&own)) => own,
        _ => of_the_system.to_string(),
    }
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = "/Users/ann";

    fn there(_: &Path) -> bool {
        true
    }

    #[cfg(target_os = "macos")]
    fn nowhere(_: &Path) -> bool {
        false
    }

    #[test]
    fn a_named_socket_is_always_the_one() {
        assert_eq!(
            socket(Some("/tmp/x.sock"), Some(Path::new(HOME)), there),
            "/tmp/x.sock"
        );
        assert_eq!(
            configuration(Some("/tmp/x.yaml"), Some(Path::new(HOME)), there),
            "/tmp/x.yaml"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn on_a_mac_the_socket_of_the_account_is_chosen_when_it_is_there() {
        assert_eq!(
            socket(None, Some(Path::new(HOME)), there),
            "/Users/ann/.local/state/vigil/vigil.sock"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn on_a_mac_without_a_socket_of_the_account_the_system_one_is_chosen() {
        assert_eq!(
            socket(None, Some(Path::new(HOME)), nowhere),
            "/var/run/vigil/vigil.sock"
        );
        assert_eq!(socket(None, None, there), "/var/run/vigil/vigil.sock");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn on_linux_the_home_is_never_looked_at() {
        assert_eq!(
            socket(None, Some(Path::new(HOME)), |_| panic!(
                "a file was looked for"
            )),
            "/run/vigil/vigil.sock"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn suppress_writes_to_the_configuration_of_the_account_when_it_is_there() {
        assert_eq!(
            configuration(None, Some(Path::new(HOME)), there),
            "/Users/ann/.config/vigil/vigil.yaml"
        );
        assert_eq!(
            configuration(None, Some(Path::new(HOME)), nowhere),
            "/usr/local/etc/vigil/vigil.yaml"
        );
    }
}
