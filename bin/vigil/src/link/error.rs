use std::fmt;

use vigil_config::Homebrew;

const ABSENT_UNDER_SYSTEMD: &[&str] = &[
    "Is the agent running?  systemctl status vigild",
    "Is this the socket it opened?  the socket_path line of its configuration",
    "Point the console at another one:  vigil ui --socket /path/to/vigil.sock",
];

const ABSENT_UNDER_LAUNCHD: &[&str] = &[
    "Is the agent running?  launchctl print system/vigil.vigild",
    "Is this the socket it opened?  the socket_path line of its configuration",
    "Point the console at another one:  vigil ui --socket /path/to/vigil.sock",
];

const ABSENT_UNDER_AN_ACCOUNT: &[&str] = &[
    "Is the agent running?  launchctl print gui/$(id -u)/vigil.vigild",
    "Is this the socket it opened?  the socket_path line of its configuration",
    "Point the console at another one:  vigil ui --socket /path/to/vigil.sock",
];

const ABSENT_UNDER_HOMEBREW: &[&str] = &[
    "Is the agent running?  brew services info vigil",
    "Not set up yet?  vigild configure, then brew services start vigil",
    "Point the console at another one:  vigil ui --socket /path/to/vigil.sock",
];

const OWN_AGENT: &str = "Or an agent of your own, without sudo:  vigild configure";

const ACCOUNT_SOCKET_TAIL: &str = "/.local/state/vigil/vigil.sock";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trouble {
    pub path: String,
    pub kind: TroubleKind,
    pub cause: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TroubleKind {
    Absent,
    Forbidden,
    Unreadable,
}

impl Trouble {
    pub fn new(path: &str, kind: TroubleKind, cause: impl Into<String>) -> Self {
        Trouble {
            path: path.to_string(),
            kind,
            cause: cause.into(),
        }
    }

    pub fn headline(&self) -> String {
        match self.kind {
            TroubleKind::Absent => {
                format!("The agent is not answering at {}.", self.path)
            }
            TroubleKind::Forbidden => {
                format!("This user may not open {}.", self.path)
            }
            TroubleKind::Unreadable => {
                format!("The agent at {} answered something unreadable.", self.path)
            }
        }
    }

    pub fn what_to_try(&self) -> Vec<String> {
        let homebrew = std::env::current_exe()
            .and_then(std::fs::canonicalize)
            .ok()
            .and_then(|program| Homebrew::of(&program))
            .is_some();
        self.what_to_try_under(homebrew)
    }

    fn what_to_try_under(&self, homebrew: bool) -> Vec<String> {
        let mac = cfg!(target_os = "macos");
        let mut said: Vec<String> = match self.kind {
            TroubleKind::Absent => match (mac, self.is_the_socket_of_an_account(), homebrew) {
                (true, true, true) => ABSENT_UNDER_HOMEBREW,
                (true, true, false) => ABSENT_UNDER_AN_ACCOUNT,
                (true, false, _) => ABSENT_UNDER_LAUNCHD,
                (false, _, _) => ABSENT_UNDER_SYSTEMD,
            }
            .iter()
            .map(|line| line.to_string())
            .collect(),
            TroubleKind::Forbidden => vec![
                "The socket is root-only, by design.".to_string(),
                "Open the console as the same user the agent runs as:  sudo vigil ui".to_string(),
            ],
            TroubleKind::Unreadable => vec![
                "Version mismatch between the agent and this console.".to_string(),
                "Both ship in one package: install them together.".to_string(),
            ],
        };
        if mac && matches!(self.kind, TroubleKind::Absent | TroubleKind::Forbidden) {
            said.push(OWN_AGENT.to_string());
        }
        said
    }

    fn is_the_socket_of_an_account(&self) -> bool {
        self.path.ends_with(ACCOUNT_SOCKET_TAIL)
    }
}

impl fmt::Display for Trouble {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.headline(), self.cause)
    }
}

impl std::error::Error for Trouble {}

#[cfg(test)]
mod tests {
    use vigil_config::{Installation, Service};

    use super::*;

    #[test]
    fn an_agent_that_is_not_answering_is_looked_for_with_the_service_manager_of_this_system() {
        assert!(ABSENT_UNDER_SYSTEMD[0].ends_with(Service::SYSTEMD.status));
        assert!(ABSENT_UNDER_LAUNCHD[0].ends_with(Service::LAUNCHD.status));

        let said = Trouble::new("/x", TroubleKind::Absent, "gone").what_to_try();
        assert!(
            said[0].ends_with(Installation::here().service.status),
            "a Mac has no systemctl, and a hint that names one sends the reader nowhere: {said:?}"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn a_stopped_agent_of_an_account_is_looked_for_in_its_gui_domain() {
        let said = Trouble::new(
            "/Users/ann/.local/state/vigil/vigil.sock",
            TroubleKind::Absent,
            "gone",
        )
        .what_to_try();

        assert_eq!(
            said[0],
            "Is the agent running?  launchctl print gui/$(id -u)/vigil.vigild"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn a_stopped_agent_of_a_homebrew_install_is_looked_for_with_brew_services() {
        let said = Trouble::new(
            "/Users/ann/.local/state/vigil/vigil.sock",
            TroubleKind::Absent,
            "gone",
        )
        .what_to_try_under(true);

        assert_eq!(said[0], "Is the agent running?  brew services info vigil");
        assert!(
            said.iter()
                .any(|line| line.ends_with("brew services start vigil")),
            "{said:?}"
        );
        assert!(ABSENT_UNDER_HOMEBREW[0].ends_with(vigil_config::SERVICE_STATUS));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn a_mac_is_told_it_can_run_an_agent_of_its_own_without_sudo() {
        for kind in [TroubleKind::Absent, TroubleKind::Forbidden] {
            let said = Trouble::new("/var/run/vigil/vigil.sock", kind, "x").what_to_try();

            assert!(said.contains(&OWN_AGENT.to_string()), "{said:?}");
        }
    }

    #[test]
    fn every_hint_fits_eighty_columns() {
        for kind in [
            TroubleKind::Absent,
            TroubleKind::Forbidden,
            TroubleKind::Unreadable,
        ] {
            for path in [
                "/run/vigil/vigil.sock",
                "/Users/ann/.local/state/vigil/vigil.sock",
            ] {
                let trouble = Trouble::new(path, kind, "x");
                let lines = trouble
                    .what_to_try()
                    .into_iter()
                    .chain(trouble.what_to_try_under(true));
                for line in lines {
                    assert!(line.chars().count() <= 80, "{line}");
                }
            }
        }
    }

    #[test]
    fn on_linux_the_hints_name_no_agent_of_an_account() {
        if cfg!(target_os = "macos") {
            return;
        }
        let said = Trouble::new("/run/vigil/vigil.sock", TroubleKind::Absent, "x").what_to_try();

        assert_eq!(said.len(), 3);
        assert!(!said.join(" ").contains("without sudo"));
    }
}
