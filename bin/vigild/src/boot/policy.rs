use crate::Config;
use crate::types::Policy;

pub fn of(config: &Config) -> Policy {
    let policy = Policy::new(config.every_suppression());
    for line in said(config) {
        eprintln!("{line}");
    }
    if policy.suppression_count() > 0 {
        for description in policy.describe_suppressions() {
            eprintln!("    {description}");
        }
    }
    policy
}

fn said(config: &Config) -> Vec<String> {
    let mut said = Vec::new();
    if !config.suppressions.is_empty() {
        said.push(format!(
            "  suppressions: {} from the configuration",
            config.suppressions.len()
        ));
    }
    for source in &config.apart.suppressions {
        said.push(format!(
            "  suppressions: {} from {}",
            source.suppressions.len(),
            source.path.display()
        ));
    }
    if let Some(at) = &config.apart.suppressions_at
        && !at.exists()
    {
        said.push(format!(
            "  suppressions: {} is not there, so nothing is silenced from it; the console makes \
             it the first time it silences something",
            at.display()
        ));
    }
    said
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use vigil_config::{Source, Suppression};

    use super::*;

    fn silencing(key: &str) -> Suppression {
        Suppression {
            finding_key: Some(key.into()),
            reason: "expected here".into(),
            ..Suppression::default()
        }
    }

    #[test]
    fn what_the_configuration_says_and_what_each_file_says_are_all_in_force_together() {
        let mut config = Config {
            suppressions: vec![silencing("user|group|docker")],
            ..Config::default()
        };
        config.apart.suppressions = vec![Source {
            path: PathBuf::from("/etc/vigil/suppressions/console.yaml"),
            suppressions: vec![silencing("port.listen|tcp|0.0.0.0:8080")],
        }];

        let policy = of(&config);

        assert_eq!(policy.suppression_count(), 2);
        let said = said(&config).join("\n");
        assert!(said.contains("1 from the configuration"), "{said}");
        assert!(
            said.contains("1 from /etc/vigil/suppressions/console.yaml"),
            "an operator reading the start-up log learns which file to open: {said}"
        );
    }

    #[test]
    fn a_directory_the_configuration_names_and_nobody_made_is_said_rather_than_refused() {
        let mut config = Config::default();
        config.apart.suppressions_at = Some(PathBuf::from("/nonexistent/vigil/suppressions"));

        let said = said(&config).join("\n");

        assert!(
            said.contains("/nonexistent/vigil/suppressions is not there"),
            "{said}"
        );
    }
}
