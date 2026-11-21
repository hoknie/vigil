use std::path::Path;

use super::account::Account;
use super::homebrew::{Homebrew, SERVICE_RESTART, SERVICE_START, SERVICE_STATUS};
use super::installation::Installation;
use crate::helpers::directories::{directory_renamed, parent_of};

pub const JOB_LABEL: &str = "vigil.vigild";

pub const NO_HOME: &str = "the home directory of this account is not known (HOME)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub configuration: String,
    pub configuration_directory: String,
    pub state_directory: String,
    pub socket: String,
    pub log_directory: String,
    pub start: String,
    pub restart: String,
    pub status: String,
    pub per_account: bool,
    pub launch_agent: Option<String>,
}

impl Layout {
    pub fn system(installation: Installation) -> Layout {
        Layout {
            configuration: installation.configuration.to_string(),
            configuration_directory: installation.configuration_directory.to_string(),
            state_directory: installation.state_directory.to_string(),
            socket: installation.socket.to_string(),
            log_directory: installation.log_directory.to_string(),
            start: installation.service.start.to_string(),
            restart: installation.service.restart.to_string(),
            status: installation.service.status.to_string(),
            per_account: false,
            launch_agent: None,
        }
    }

    pub fn for_account(installation: Installation, account: &Account) -> Result<Layout, String> {
        if account.uid == 0 || installation != Installation::MACOS {
            return Ok(Layout::system(installation));
        }
        let home = account
            .home
            .as_deref()
            .and_then(home_of)
            .ok_or_else(|| NO_HOME.to_string())?;
        let uid = account.uid;
        let launch_agent = launch_agent_in(home);
        Ok(Layout {
            configuration: configuration_in(home),
            configuration_directory: format!("{home}/.config/vigil"),
            state_directory: format!("{home}/.local/state/vigil"),
            socket: socket_in(home),
            log_directory: format!("{home}/Library/Logs/vigil"),
            start: format!("launchctl bootstrap gui/{uid} {launch_agent}"),
            restart: format!("launchctl kickstart -k gui/{uid}/{JOB_LABEL}"),
            status: format!("launchctl print gui/{uid}/{JOB_LABEL}"),
            per_account: true,
            launch_agent: Some(launch_agent),
        })
    }

    pub fn under(self, homebrew: Option<&Homebrew>) -> Layout {
        match (homebrew, self.per_account) {
            (Some(_), true) => Layout {
                start: SERVICE_START.to_string(),
                restart: SERVICE_RESTART.to_string(),
                status: SERVICE_STATUS.to_string(),
                launch_agent: None,
                ..self
            },
            _ => self,
        }
    }

    pub fn account_launch_agent(home: &Path) -> Option<String> {
        home_of(home).map(launch_agent_in)
    }

    pub fn socket_under(home: &Path) -> Option<String> {
        home_of(home).map(socket_in)
    }

    pub fn configuration_under(home: &Path) -> Option<String> {
        home_of(home).map(configuration_in)
    }

    pub fn moved_from(&self, shipped: &Installation, text: &str) -> String {
        let mut moved = text.to_string();
        for (from, to) in [
            (shipped.state_directory, self.state_directory.as_str()),
            (shipped.log_directory, self.log_directory.as_str()),
            (parent_of(shipped.socket), parent_of(&self.socket)),
            (
                shipped.configuration_directory,
                self.configuration_directory.as_str(),
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

fn home_of(home: &Path) -> Option<&str> {
    home.is_absolute()
        .then_some(home)
        .and_then(Path::to_str)
        .map(|home| home.trim_end_matches('/'))
        .filter(|home| !home.is_empty())
}

fn launch_agent_in(home: &str) -> String {
    format!("{home}/Library/LaunchAgents/{JOB_LABEL}.plist")
}

fn socket_in(home: &str) -> String {
    format!("{home}/.local/state/vigil/vigil.sock")
}

fn configuration_in(home: &str) -> String {
    format!("{home}/.config/vigil/vigil.yaml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn account(uid: u32, home: Option<&str>) -> Account {
        Account {
            uid,
            home: home.map(PathBuf::from),
        }
    }

    #[test]
    fn on_macos_an_account_that_is_not_root_keeps_everything_under_its_home() {
        let layout =
            Layout::for_account(Installation::MACOS, &account(501, Some("/Users/ann"))).unwrap();

        assert_eq!(layout.configuration, "/Users/ann/.config/vigil/vigil.yaml");
        assert_eq!(layout.configuration_directory, "/Users/ann/.config/vigil");
        assert_eq!(layout.state_directory, "/Users/ann/.local/state/vigil");
        assert_eq!(layout.socket, "/Users/ann/.local/state/vigil/vigil.sock");
        assert_eq!(layout.log_directory, "/Users/ann/Library/Logs/vigil");
        assert!(layout.per_account);
        assert_eq!(
            layout.launch_agent.as_deref(),
            Some("/Users/ann/Library/LaunchAgents/vigil.vigild.plist")
        );
    }

    #[test]
    fn root_and_linux_keep_the_places_the_packages_install_to() {
        for (installation, uid) in [
            (Installation::MACOS, 0),
            (Installation::LINUX, 0),
            (Installation::LINUX, 501),
        ] {
            let layout =
                Layout::for_account(installation, &account(uid, Some("/Users/ann"))).unwrap();

            assert_eq!(layout, Layout::system(installation));
            assert!(!layout.per_account);
            assert_eq!(layout.configuration, installation.configuration);
            assert_eq!(layout.socket, installation.socket);
            assert_eq!(layout.start, installation.service.start);
            assert_eq!(layout.restart, installation.service.restart);
            assert_eq!(layout.status, installation.service.status);
        }
    }

    #[test]
    fn an_account_without_a_home_is_told_to_name_its_places() {
        let said = Layout::for_account(Installation::MACOS, &account(501, None)).unwrap_err();

        assert_eq!(said, NO_HOME);
    }

    #[test]
    fn a_relative_home_is_no_home() {
        for home in ["", "ann", "./ann", "/"] {
            assert!(
                Layout::for_account(Installation::MACOS, &account(501, Some(home))).is_err(),
                "{home:?}"
            );
        }
    }

    #[test]
    fn the_service_of_an_account_is_its_gui_domain() {
        let layout =
            Layout::for_account(Installation::MACOS, &account(501, Some("/Users/ann"))).unwrap();

        assert_eq!(
            layout.start,
            "launchctl bootstrap gui/501 /Users/ann/Library/LaunchAgents/vigil.vigild.plist"
        );
        assert_eq!(
            layout.restart,
            "launchctl kickstart -k gui/501/vigil.vigild"
        );
        assert_eq!(layout.status, "launchctl print gui/501/vigil.vigild");
    }

    #[test]
    fn a_shipped_file_moved_to_an_account_names_its_home() {
        let layout =
            Layout::for_account(Installation::MACOS, &account(501, Some("/Users/ann"))).unwrap();
        let shipped = "state_dir: /var/lib/vigil\n\
                       socket_path: /run/vigil/vigil.sock\n\
                       collectors_path: /etc/vigil/collectors\n\
                       #     path: /var/log/vigil/findings.ndjson\n";

        assert_eq!(
            layout.moved_from(&Installation::LINUX, shipped),
            "state_dir: /Users/ann/.local/state/vigil\n\
             socket_path: /Users/ann/.local/state/vigil/vigil.sock\n\
             collectors_path: /Users/ann/.config/vigil/collectors\n\
             #     path: /Users/ann/Library/Logs/vigil/findings.ndjson\n"
        );
    }

    #[test]
    fn the_places_of_an_account_are_known_from_its_home_alone() {
        assert_eq!(
            Layout::socket_under(Path::new("/Users/ann")).as_deref(),
            Some("/Users/ann/.local/state/vigil/vigil.sock")
        );
        assert_eq!(
            Layout::configuration_under(Path::new("/Users/ann/")).as_deref(),
            Some("/Users/ann/.config/vigil/vigil.yaml")
        );
        assert_eq!(Layout::socket_under(Path::new("ann")), None);
    }

    #[test]
    fn the_socket_of_an_account_fits_a_unix_socket_path() {
        let name = "a".repeat(32);
        let home = format!("/Users/{name}");
        let layout = Layout::for_account(Installation::MACOS, &account(501, Some(&home))).unwrap();

        assert!(layout.socket.len() < 104, "{}", layout.socket);
    }

    #[test]
    fn under_homebrew_an_account_starts_with_brew_services_and_gets_no_launch_agent() {
        let brew = Homebrew {
            prefix: "/opt/homebrew".into(),
        };
        let layout = Layout::for_account(Installation::MACOS, &account(501, Some("/Users/ann")))
            .unwrap()
            .under(Some(&brew));

        assert_eq!(layout.start, "brew services start vigil");
        assert_eq!(layout.restart, "brew services restart vigil");
        assert_eq!(layout.status, "brew services info vigil");
        assert_eq!(layout.launch_agent, None);
        assert!(layout.per_account);
        assert_eq!(layout.configuration, "/Users/ann/.config/vigil/vigil.yaml");
        assert_eq!(layout.socket, "/Users/ann/.local/state/vigil/vigil.sock");
    }

    #[test]
    fn under_homebrew_root_and_linux_keep_their_layout() {
        let brew = Homebrew {
            prefix: "/opt/homebrew".into(),
        };
        for (installation, uid) in [
            (Installation::MACOS, 0),
            (Installation::LINUX, 0),
            (Installation::LINUX, 501),
        ] {
            let layout =
                Layout::for_account(installation, &account(uid, Some("/Users/ann"))).unwrap();

            assert_eq!(layout.clone().under(Some(&brew)), layout);
        }
    }

    #[test]
    fn outside_homebrew_the_layout_is_the_one_it_was() {
        let layout =
            Layout::for_account(Installation::MACOS, &account(501, Some("/Users/ann"))).unwrap();

        assert_eq!(layout.clone().under(None), layout);
        assert_eq!(
            Layout::account_launch_agent(Path::new("/Users/ann")),
            layout.launch_agent
        );
    }
}
