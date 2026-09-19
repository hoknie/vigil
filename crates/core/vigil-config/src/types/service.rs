#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Service {
    pub manager: &'static str,
    pub start: &'static str,
    pub restart: &'static str,
    pub status: &'static str,
}

impl Service {
    pub const SYSTEMD: Service = Service {
        manager: "systemd",
        start: "systemctl enable --now vigild",
        restart: "systemctl try-restart vigild.service",
        status: "systemctl status vigild",
    };

    pub const LAUNCHD: Service = Service {
        manager: "launchd",
        start: "launchctl bootstrap system /Library/LaunchDaemons/vigil.vigild.plist",
        restart: "launchctl kickstart -k system/vigil.vigild",
        status: "launchctl print system/vigil.vigild",
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_linux_the_daemon_is_a_systemd_unit_called_vigild() {
        assert_eq!(Service::SYSTEMD.start, "systemctl enable --now vigild");
        assert_eq!(
            Service::SYSTEMD.restart,
            "systemctl try-restart vigild.service"
        );
        assert_eq!(Service::SYSTEMD.status, "systemctl status vigild");
    }

    #[test]
    fn on_macos_the_daemon_is_the_launchd_job_the_package_installs_as_vigil_vigild() {
        for said in [
            Service::LAUNCHD.start,
            Service::LAUNCHD.restart,
            Service::LAUNCHD.status,
        ] {
            assert!(said.starts_with("launchctl "), "{said}");
            assert!(said.contains("vigil.vigild"), "{said}");
        }
        assert!(
            Service::LAUNCHD
                .start
                .ends_with("/Library/LaunchDaemons/vigil.vigild.plist"),
            "bootstrap takes the path of the plist the package installs, not the label"
        );
    }
}
