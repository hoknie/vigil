use crate::Config;

pub fn switched_off_reason(config: &Config, name: &str) -> String {
    let because = match (&config.apart.collectors_at, config.apart.block(name)) {
        (None, _) => "not named in `collectors:`".to_string(),
        (Some(_), Some(block)) => format!("`enabled: false` in {}", block.file.display()),
        (Some(at), None) => format!("no block for it in {}", at.display()),
    };
    format!(
        "{because}, so nothing is watching {}: switched off, not failing",
        crate::modules::subject_of(name).unwrap_or("what it watches")
    )
}

pub fn switched_off_reasons(config: &Config, names: &[String]) -> Vec<(String, String)> {
    names
        .iter()
        .map(|name| (name.clone(), switched_off_reason(config, name)))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use vigil_config::Block;

    use super::*;

    fn apart(at: &Path, blocks: Vec<Block>) -> Config {
        let mut config = Config::default();
        config.apart.collectors_at = Some(at.to_path_buf());
        config.apart.collectors = blocks;
        config
    }

    #[test]
    fn a_collector_whose_block_says_enabled_false_is_off_by_that_line_of_that_file() {
        let config = apart(
            Path::new("/etc/vigil/collectors"),
            vec![Block {
                name: "files".to_string(),
                file: PathBuf::from("/etc/vigil/collectors/files.yaml"),
                enabled: false,
                schedule: None,
                settings: serde_yaml::Mapping::new(),
            }],
        );

        let reason = switched_off_reason(&config, "files");

        assert!(
            reason.starts_with("`enabled: false` in /etc/vigil/collectors/files.yaml"),
            "{reason}"
        );
        assert!(reason.contains("not failing"), "{reason}");
        assert!(!reason.contains("`collectors:`"), "{reason}");
    }

    #[test]
    fn a_collector_with_no_block_is_off_because_the_collectors_path_holds_none_for_it() {
        let config = apart(Path::new("/etc/vigil/collectors"), Vec::new());

        let reason = switched_off_reason(&config, "launches");

        assert!(
            reason.starts_with("no block for it in /etc/vigil/collectors"),
            "{reason}"
        );
        assert!(reason.contains("what people run"), "{reason}");
    }

    #[test]
    fn a_file_of_the_former_layout_is_told_about_in_the_words_it_was_written_in() {
        let reason = switched_off_reason(&Config::default(), "launches");

        assert!(reason.starts_with("not named in `collectors:`"), "{reason}");
    }
}
