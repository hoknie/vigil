use std::path::Path;

use vigil_collect::Health;
use vigil_model::{Finding, Host};

use crate::Config;
use crate::helpers::agent_finding;
use crate::loops::Watch;
use crate::socket::switched_off_reason;

pub struct Greeting<'a> {
    pub config_path: &'a str,
    pub config: &'a Config,
    pub host: &'a Host,
    pub watches: &'a [Watch],
    pub switched_off: &'a [String],
    pub reporters: usize,
    pub damaged: usize,
}

impl Greeting<'_> {
    pub fn say(&self) {
        eprintln!(
            "vigild {} — host {}, install {}, state {}, {} reporter(s)",
            env!("CARGO_PKG_VERSION"),
            self.host.host_id,
            self.host.install_id,
            self.config.state_dir,
            self.reporters
        );
        self.say_reporters();
        self.say_collectors();
        self.say_health();
        if self.damaged > 0 {
            eprintln!(
                "  history: {} unreadable line(s) in the findings journal, skipped on open",
                self.damaged
            );
        }
    }

    pub fn findings(&self) -> Vec<Finding> {
        let mut findings = Vec::new();

        for watch in self.watches {
            match watch.health() {
                Health::Ok => {}
                Health::Degraded(detail) => findings.push(agent_finding::collector_degraded(
                    watch.name(),
                    &detail,
                    false,
                )),
                Health::Unavailable(detail) => findings.push(agent_finding::collector_degraded(
                    watch.name(),
                    &detail,
                    true,
                )),
            }
        }
        if self.damaged > 0 {
            findings.push(agent_finding::store_damaged(
                "findings",
                "the local findings history",
                self.damaged,
            ));
        }

        findings
    }

    fn say_reporters(&self) {
        for file in &self.config.apart.reporters {
            eprintln!(
                "  reporters: {} from {}",
                file.receivers.len(),
                file.path.display()
            );
        }
        if let Some(at) = &self.config.apart.reporters_at
            && !at.exists()
        {
            eprintln!(
                "  reporters: {} is not there, so no reporter is read from it",
                at.display()
            );
        }
    }

    fn say_collectors(&self) {
        let watched: Vec<&str> = self.watches.iter().map(|watch| watch.name()).collect();
        for line in collectors(self.config_path, self.config, &watched, self.switched_off) {
            eprintln!("{line}");
        }
    }

    fn say_health(&self) {
        for watch in self.watches {
            match watch.health() {
                Health::Ok => eprintln!("  collector {}: ok", watch.name()),
                Health::Degraded(detail) => {
                    eprintln!("  collector {}: degraded — {detail}", watch.name())
                }
                Health::Unavailable(detail) => {
                    eprintln!("  collector {}: unavailable — {detail}", watch.name())
                }
            }
        }
    }
}

fn collectors(
    config_path: &str,
    config: &Config,
    watched: &[&str],
    switched_off: &[String],
) -> Vec<String> {
    let mut said = Vec::new();
    let at = config.apart.collectors_at.as_deref();
    if let Some(at) = at {
        said.push(format!("  collectors: read from {}", at.display()));
    }

    said.push(match (&config.collectors, at) {
        (None, _) => format!(
            "  collectors: all {} this build has. `collectors:` names none, which means all of them",
            watched.len()
        ),
        (Some(_), Some(at)) if watched.is_empty() => format!(
            "  collectors: NONE. {}: nothing is watched. History is kept and the console is answered",
            nothing_in(at, config)
        ),
        (Some(_), None) if watched.is_empty() => format!(
            "  collectors: NONE. `collectors:` in {config_path} is empty: nothing is watched. History is kept and the console is answered"
        ),
        (Some(_), _) => format!(
            "  collectors: {} ({} of {}, {})",
            watched.join(", "),
            watched.len(),
            watched.len() + switched_off.len(),
            match at {
                Some(_) => "each with a block that does not say `enabled: false`",
                None => "named in `collectors:`",
            }
        ),
    });

    for name in switched_off {
        said.push(format!(
            "  collector {name}: off — {}",
            switched_off_reason(config, name)
        ));
    }
    said
}

fn nothing_in(at: &Path, config: &Config) -> String {
    if !at.exists() {
        return format!("{} is not there", at.display());
    }
    match config.apart.collectors.is_empty() {
        true => format!("{} holds no block of a collector", at.display()),
        false => format!("every block in {} says `enabled: false`", at.display()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use vigil_config::Block;

    use super::*;

    fn block(name: &str, file: &str, enabled: bool) -> Block {
        Block {
            name: name.to_string(),
            file: PathBuf::from(file),
            enabled,
            schedule: None,
            settings: serde_yaml::Mapping::new(),
        }
    }

    fn apart(at: &Path, blocks: Vec<Block>) -> Config {
        let mut config = Config {
            collectors: Some(
                blocks
                    .iter()
                    .filter(|block| block.enabled)
                    .map(|block| block.name.clone())
                    .collect(),
            ),
            ..Config::default()
        };
        config.apart.collectors_at = Some(at.to_path_buf());
        config.apart.collectors = blocks;
        config
    }

    #[test]
    fn where_the_collectors_are_read_from_and_which_blocks_are_off_is_said_at_start_up() {
        let config = apart(
            Path::new("/etc/vigil/collectors"),
            vec![
                block("users", "/etc/vigil/collectors/users.yaml", true),
                block("files", "/etc/vigil/collectors/files.yaml", false),
            ],
        );

        let said = collectors(
            "/etc/vigil/vigil.yaml",
            &config,
            &["users"],
            &["files".to_string(), "launches".to_string()],
        )
        .join("\n");

        assert!(said.contains("read from /etc/vigil/collectors"), "{said}");
        assert!(
            said.contains(
                "collector files: off — `enabled: false` in /etc/vigil/collectors/files.yaml"
            ),
            "an operator reading the start-up learns which line in which file switched it off: {said}"
        );
        assert!(
            said.contains("collector launches: off — no block for it in /etc/vigil/collectors"),
            "{said}"
        );
        assert!(!said.contains("`collectors:`"), "{said}");
    }

    #[test]
    fn a_directory_that_is_not_there_and_one_with_nothing_in_it_are_each_said_in_words_of_their_own()
     {
        let directory =
            std::env::temp_dir().join(format!("vigild-greeting-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("a directory");

        let absent = collectors(
            "/etc/vigil/vigil.yaml",
            &apart(&directory.join("collectors"), Vec::new()),
            &[],
            &[],
        )
        .join("\n");
        let empty = collectors(
            "/etc/vigil/vigil.yaml",
            &apart(&directory, Vec::new()),
            &[],
            &[],
        )
        .join("\n");
        let all_off = collectors(
            "/etc/vigil/vigil.yaml",
            &apart(&directory, vec![block("users", "users.yaml", false)]),
            &[],
            &["users".to_string()],
        )
        .join("\n");

        assert!(
            absent.contains("NONE") && absent.contains("is not there"),
            "{absent}"
        );
        assert!(
            empty.contains("NONE") && empty.contains("holds no block"),
            "{empty}"
        );
        assert!(
            all_off.contains("NONE") && all_off.contains("enabled: false"),
            "{all_off}"
        );
        assert_ne!(absent, empty);
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_file_of_the_former_layout_is_still_told_about_in_the_words_it_was_written_in() {
        let config = Config {
            collectors: Some(Vec::new()),
            ..Config::default()
        };

        let said = collectors("/etc/vigil/vigil.yaml", &config, &[], &[]).join("\n");

        assert!(
            said.contains("`collectors:` in /etc/vigil/vigil.yaml is empty"),
            "{said}"
        );
        assert!(!said.contains("read from"), "{said}");
    }
}
