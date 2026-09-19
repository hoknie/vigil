use std::path::{Path, PathBuf};

pub const SYSTEMD: &str = "/run/systemd/system";

pub const LAUNCH_DAEMONS: &str = "/Library/LaunchDaemons";

const UNITS: &str = "/usr/lib/systemd/system";

const OURS: &str = "vigil-";

const OUR_LABELS: &str = "vigil.";

const DOMAIN: &str = "system";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Manager {
    Systemd,
    Launchd,
}

impl Manager {
    pub const fn here() -> Manager {
        match cfg!(target_os = "macos") {
            true => Manager::Launchd,
            false => Manager::Systemd,
        }
    }

    pub fn job(self, unit: &str) -> String {
        match self {
            Manager::Systemd => unit.to_string(),
            Manager::Launchd => {
                let stem = unit.rsplit_once('.').map_or(unit, |(stem, _)| stem);
                match stem.strip_prefix(OURS) {
                    Some(rest) => format!("{OUR_LABELS}{rest}"),
                    None => stem.to_string(),
                }
            }
        }
    }

    pub fn installed_at(self, unit: &str) -> PathBuf {
        match self {
            Manager::Systemd => Path::new(UNITS).join(Manager::service_of(unit)),
            Manager::Launchd => Path::new(LAUNCH_DAEMONS).join(format!("{}.plist", self.job(unit))),
        }
    }

    pub fn missing(self, unit: &str) -> Option<String> {
        match self {
            Manager::Systemd => (!Path::new(SYSTEMD).is_dir()).then(|| {
                format!(
                    "there is no systemd on this host ({SYSTEMD} is not a directory): run what {} \
                     runs on a period of your own",
                    self.installed_at(unit).display()
                )
            }),
            Manager::Launchd => {
                let plist = self.installed_at(unit);
                (!plist.is_file()).then(|| {
                    format!(
                        "{} is not there: the package installs it, and without it launchd has \
                         nothing to start",
                        plist.display()
                    )
                })
            }
        }
    }

    pub fn asking(self, unit: &str) -> Vec<String> {
        match self {
            Manager::Systemd => Manager::words(&["is-enabled", unit]),
            Manager::Launchd => Manager::words(&["print", &self.target(unit)]),
        }
    }

    pub fn enabling(self, unit: &str, loaded: bool) -> Vec<Vec<String>> {
        match self {
            Manager::Systemd => vec![Manager::words(&["enable", "--now", unit])],
            Manager::Launchd => {
                let mut steps = vec![Manager::words(&["enable", &self.target(unit)])];
                if !loaded {
                    steps.push(Manager::words(&[
                        "bootstrap",
                        DOMAIN,
                        &self.installed_at(unit).display().to_string(),
                    ]));
                }
                steps
            }
        }
    }

    pub fn disabling(self, unit: &str, loaded: bool) -> Vec<Vec<String>> {
        match self {
            Manager::Systemd => vec![Manager::words(&["disable", "--now", unit])],
            Manager::Launchd => {
                let mut steps = Vec::new();
                if loaded {
                    steps.push(Manager::words(&["bootout", &self.target(unit)]));
                }
                steps.push(Manager::words(&["disable", &self.target(unit)]));
                steps
            }
        }
    }

    pub fn masking_undone(self, unit: &str) -> Option<String> {
        match self {
            Manager::Systemd => Some(format!("systemctl unmask {unit}")),
            Manager::Launchd => None,
        }
    }

    fn target(self, unit: &str) -> String {
        format!("{DOMAIN}/{}", self.job(unit))
    }

    fn service_of(unit: &str) -> String {
        match unit.strip_suffix(".timer") {
            Some(stem) => format!("{stem}.service"),
            None => unit.to_string(),
        }
    }

    fn words(said: &[&str]) -> Vec<String> {
        said.iter().map(|word| word.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(said: &[&str]) -> Vec<String> {
        said.iter().map(|word| word.to_string()).collect()
    }

    #[test]
    fn the_build_for_a_mac_asks_launchd_and_every_other_build_asks_systemd() {
        assert_eq!(
            Manager::here(),
            match cfg!(target_os = "macos") {
                true => Manager::Launchd,
                false => Manager::Systemd,
            }
        );
    }

    #[test]
    fn a_timer_of_ours_is_the_launchd_job_the_package_installs_under_the_same_name() {
        assert_eq!(
            Manager::Launchd.job("vigil-firewall.timer"),
            "vigil.firewall"
        );
        assert_eq!(
            Manager::Launchd.job("vigil-containers.timer"),
            "vigil.containers"
        );
        assert_eq!(
            Manager::Launchd.installed_at("vigil-firewall.timer"),
            PathBuf::from("/Library/LaunchDaemons/vigil.firewall.plist")
        );
        assert_eq!(
            Manager::Systemd.job("vigil-firewall.timer"),
            "vigil-firewall.timer"
        );
    }

    #[test]
    fn on_linux_a_reading_is_switched_with_one_systemctl_word_that_also_starts_or_stops_it() {
        assert_eq!(
            Manager::Systemd.enabling("vigil-firewall.timer", false),
            vec![words(&["enable", "--now", "vigil-firewall.timer"])]
        );
        assert_eq!(
            Manager::Systemd.disabling("vigil-firewall.timer", true),
            vec![words(&["disable", "--now", "vigil-firewall.timer"])]
        );
    }

    #[test]
    fn on_macos_enabling_clears_a_disabled_job_and_loads_it_unless_it_is_loaded_already() {
        assert_eq!(
            Manager::Launchd.enabling("vigil-firewall.timer", false),
            vec![
                words(&["enable", "system/vigil.firewall"]),
                words(&[
                    "bootstrap",
                    "system",
                    "/Library/LaunchDaemons/vigil.firewall.plist"
                ]),
            ]
        );
        assert_eq!(
            Manager::Launchd.enabling("vigil-firewall.timer", true),
            vec![words(&["enable", "system/vigil.firewall"])],
            "bootstrap of a loaded job is refused by launchd with an error that reads like a \
             failure, and the job is already doing what was asked"
        );
    }

    #[test]
    fn on_macos_disabling_unloads_the_job_and_keeps_it_from_loading_at_the_next_boot() {
        assert_eq!(
            Manager::Launchd.disabling("vigil-containers.timer", true),
            vec![
                words(&["bootout", "system/vigil.containers"]),
                words(&["disable", "system/vigil.containers"]),
            ]
        );
        assert_eq!(
            Manager::Launchd.disabling("vigil-containers.timer", false),
            vec![words(&["disable", "system/vigil.containers"])]
        );
    }

    #[test]
    fn a_job_launchd_has_no_plist_for_is_named_by_the_file_that_is_missing() {
        let missing = Manager::Launchd
            .missing("vigil-no-such-reading.timer")
            .expect("nothing installs it");

        assert!(
            missing.contains("/Library/LaunchDaemons/vigil.no-such-reading.plist"),
            "{missing}"
        );
    }

    #[test]
    fn a_host_without_systemd_is_told_which_unit_file_holds_what_to_run_instead() {
        let Some(missing) = Manager::Systemd.missing("vigil-firewall.timer") else {
            return;
        };

        assert!(
            missing.contains("/usr/lib/systemd/system/vigil-firewall.service"),
            "{missing}"
        );
    }

    #[test]
    fn launchd_has_no_mask_and_nothing_tells_an_operator_to_unmask_there() {
        assert_eq!(
            Manager::Launchd.masking_undone("vigil-firewall.timer"),
            None
        );
        assert_eq!(
            Manager::Systemd
                .masking_undone("vigil-firewall.timer")
                .as_deref(),
            Some("systemctl unmask vigil-firewall.timer")
        );
    }

    #[test]
    fn every_unit_a_module_names_is_a_launchd_job_the_mac_package_installs() {
        let jobs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packaging/macos/launchd");
        let mut named = 0;

        for watched in crate::modules::watched() {
            let Some(unit) = watched.unit else {
                continue;
            };
            let plist = Manager::Launchd.installed_at(unit);
            let file = plist.file_name().expect("a file name");
            let text = std::fs::read_to_string(jobs.join(file)).unwrap_or_else(|error| {
                panic!(
                    "{}: {unit} has no job in packaging/macos/launchd, and `vigild collector \
                     {} enable` would bootstrap a file no package installs: {error}",
                    plist.display(),
                    watched.name
                )
            });
            assert!(
                text.contains(&format!("<string>{}</string>", Manager::Launchd.job(unit))),
                "{}",
                plist.display()
            );
            named += 1;
        }

        assert!(named >= 2, "only {named} unit(s) were checked");
    }
}
