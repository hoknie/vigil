const MACOS: &str = "macos";

const ACCOUNTS_NOT_ON_MACOS: &str = "the accounts of a Mac live in Directory Services, and this \
                                     agent changes accounts only with the tools of a Linux host \
                                     (useradd and its family, gpasswd, visudo, loginctl): \
                                     nothing was changed. Change it in System Settings, or with \
                                     dscl or sysadminctl";

const UNITS_NOT_ON_MACOS: &str = "what a Mac starts by itself is a launchd job, and this agent \
                                  stops, starts, disables and enables systemd units only: \
                                  nothing was touched. launchctl bootout, bootstrap, disable \
                                  and enable do it by hand";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum System {
    Linux,
    Macos,
}

impl System {
    pub fn of(family: &str) -> System {
        match family == MACOS {
            true => System::Macos,
            false => System::Linux,
        }
    }

    pub fn changes_accounts(self) -> Result<(), &'static str> {
        match self {
            System::Linux => Ok(()),
            System::Macos => Err(ACCOUNTS_NOT_ON_MACOS),
        }
    }

    pub fn controls_units(self) -> Result<(), &'static str> {
        match self {
            System::Linux => Ok(()),
            System::Macos => Err(UNITS_NOT_ON_MACOS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mac_is_named_by_the_family_the_host_description_carries_and_the_rest_is_linux() {
        assert_eq!(System::of("macos"), System::Macos);
        assert_eq!(System::of("linux"), System::Linux);
        assert_eq!(
            System::of(std::env::consts::OS),
            match cfg!(target_os = "macos") {
                true => System::Macos,
                false => System::Linux,
            }
        );
    }

    #[test]
    fn on_linux_the_console_may_change_accounts_and_control_units_once_switched_on() {
        assert_eq!(System::Linux.changes_accounts(), Ok(()));
        assert_eq!(System::Linux.controls_units(), Ok(()));
    }

    #[test]
    fn on_macos_accounts_and_units_are_refused_with_what_to_use_instead() {
        let accounts = System::Macos.changes_accounts().expect_err("not ported");
        let units = System::Macos.controls_units().expect_err("not ported");

        assert!(accounts.contains("Directory Services"), "{accounts}");
        assert!(accounts.contains("dscl"), "{accounts}");
        assert!(units.contains("launchd"), "{units}");
        assert!(units.contains("launchctl"), "{units}");
    }
}
