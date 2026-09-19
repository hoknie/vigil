use super::service::Service;
use crate::helpers::directories::{directory_renamed, parent_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Installation {
    pub configuration: &'static str,
    pub configuration_directory: &'static str,
    pub state_directory: &'static str,
    pub socket: &'static str,
    pub log_directory: &'static str,
    pub service: Service,
}

impl Installation {
    pub const LINUX: Installation = Installation {
        configuration: "/etc/vigil/vigil.yaml",
        configuration_directory: "/etc/vigil",
        state_directory: "/var/lib/vigil",
        socket: "/run/vigil/vigil.sock",
        log_directory: "/var/log/vigil",
        service: Service::SYSTEMD,
    };

    pub const MACOS: Installation = Installation {
        configuration: "/usr/local/etc/vigil/vigil.yaml",
        configuration_directory: "/usr/local/etc/vigil",
        state_directory: "/usr/local/var/lib/vigil",
        socket: "/var/run/vigil/vigil.sock",
        log_directory: "/usr/local/var/log/vigil",
        service: Service::LAUNCHD,
    };

    pub const fn here() -> Installation {
        match cfg!(target_os = "macos") {
            true => Installation::MACOS,
            false => Installation::LINUX,
        }
    }

    pub fn moved_from(&self, shipped: &Installation, text: &str) -> String {
        let mut moved = text.to_string();
        for (from, to) in [
            (shipped.state_directory, self.state_directory),
            (shipped.log_directory, self.log_directory),
            (parent_of(shipped.socket), parent_of(self.socket)),
            (
                shipped.configuration_directory,
                self.configuration_directory,
            ),
        ] {
            if from == to {
                continue;
            }
            moved = directory_renamed(&moved, from, to);
        }
        moved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_linux_the_agent_lives_where_the_deb_and_the_rpm_put_it() {
        assert_eq!(Installation::LINUX.configuration, "/etc/vigil/vigil.yaml");
        assert_eq!(Installation::LINUX.configuration_directory, "/etc/vigil");
        assert_eq!(Installation::LINUX.state_directory, "/var/lib/vigil");
        assert_eq!(Installation::LINUX.socket, "/run/vigil/vigil.sock");
        assert_eq!(Installation::LINUX.log_directory, "/var/log/vigil");
        assert_eq!(Installation::LINUX.service, Service::SYSTEMD);
    }

    #[test]
    fn on_macos_the_agent_lives_under_usr_local_because_the_system_volume_is_read_only() {
        assert_eq!(
            Installation::MACOS.configuration,
            "/usr/local/etc/vigil/vigil.yaml"
        );
        assert_eq!(
            Installation::MACOS.configuration_directory,
            "/usr/local/etc/vigil"
        );
        assert_eq!(
            Installation::MACOS.state_directory,
            "/usr/local/var/lib/vigil"
        );
        assert_eq!(
            Installation::MACOS.socket,
            "/var/run/vigil/vigil.sock",
            "/var/run is emptied at every boot on macOS as /run is on Linux, and the daemon \
             makes the directory again when it binds"
        );
        assert_eq!(
            Installation::MACOS.log_directory,
            "/usr/local/var/log/vigil"
        );
        assert_eq!(Installation::MACOS.service, Service::LAUNCHD);
    }

    #[test]
    fn the_configuration_file_lives_in_the_configuration_directory_on_every_system() {
        for installation in [Installation::LINUX, Installation::MACOS] {
            assert_eq!(
                installation
                    .configuration
                    .rsplit_once('/')
                    .map(|(directory, _)| directory),
                Some(installation.configuration_directory)
            );
        }
    }

    #[test]
    fn the_build_for_this_system_names_the_places_of_this_system() {
        let expected = match cfg!(target_os = "macos") {
            true => Installation::MACOS,
            false => Installation::LINUX,
        };

        assert_eq!(Installation::here(), expected);
    }

    #[test]
    fn a_shipped_file_moved_to_macos_names_the_places_of_macos() {
        let shipped = "state_dir: /var/lib/vigil\n\
                       socket_path: /run/vigil/vigil.sock\n\
                       collectors_path: /etc/vigil/collectors\n\
                       #     path: /var/log/vigil/findings.ndjson\n";

        assert_eq!(
            Installation::MACOS.moved_from(&Installation::LINUX, shipped),
            "state_dir: /usr/local/var/lib/vigil\n\
             socket_path: /var/run/vigil/vigil.sock\n\
             collectors_path: /usr/local/etc/vigil/collectors\n\
             #     path: /usr/local/var/log/vigil/findings.ndjson\n"
        );
    }

    #[test]
    fn a_shipped_file_moved_to_the_system_it_was_written_for_is_left_byte_for_byte() {
        let shipped = "state_dir: /var/lib/vigil\nsocket_path: /run/vigil/vigil.sock\n";

        assert_eq!(
            Installation::LINUX.moved_from(&Installation::LINUX, shipped),
            shipped
        );
    }

    #[test]
    fn a_path_that_only_begins_like_one_of_ours_is_somebody_elses_and_is_left_alone() {
        let text = "a: /var/lib/vigilance\nb: /srv/var/lib/vigil\nc: /etc/vigil-other/x\n";

        assert_eq!(
            Installation::MACOS.moved_from(&Installation::LINUX, text),
            text
        );
    }
}
