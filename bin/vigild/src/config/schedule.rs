use super::Config;

pub fn check(config: &Config) -> Result<(), String> {
    if config.interval_seconds == Some(0) {
        return Err("interval_seconds: 0 is reading with no pause between readings".to_string());
    }

    for (name, every_seconds) in &config.schedule {
        if !crate::modules::is_known(name) {
            return Err(format!(
                "schedule {name:?}: unknown collector; known: {}",
                crate::modules::names().join(", ")
            ));
        }
        if let Some(enabled) = &config.collectors
            && !enabled.iter().any(|wanted| wanted == name)
        {
            return Err(format!(
                "schedule {name:?}: a period for a collector that does not run — {name} is not named in `collectors:`"
            ));
        }
        if *every_seconds == 0 {
            return Err(format!(
                "schedule {name:?}: 0 is reading with no pause between readings"
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::load;

    #[test]
    fn an_existing_file_that_names_one_interval_keeps_every_collector_on_it() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("yesterdays-interval.yaml");
        std::fs::write(&path, "interval_seconds: 10\n").expect("write");

        let config = load(path.to_str().expect("utf-8")).expect("parses");

        assert_eq!(config.interval_seconds, Some(10));
        for name in crate::modules::names() {
            assert_eq!(
                config.every_seconds(name),
                10,
                "{name} moved off the period this file has always meant"
            );
        }
    }

    #[test]
    fn a_file_that_names_neither_gives_each_collector_its_own_period() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("no-schedule.yaml");
        std::fs::write(&path, "{}\n").expect("write");

        let config = load(path.to_str().expect("utf-8")).expect("parses");

        assert_eq!(config.interval_seconds, None);
        assert_eq!(config.every_seconds("launches"), 15);
        assert_eq!(config.every_seconds("users"), 300);
    }

    #[test]
    fn a_named_period_is_read_and_leaves_the_others_where_they_were() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("schedule.yaml");
        std::fs::write(
            &path,
            "interval_seconds: 10\nschedule:\n  persistence: 600\n",
        )
        .expect("write");

        let config = load(path.to_str().expect("utf-8")).expect("parses");

        assert_eq!(config.every_seconds("persistence"), 600);
        assert_eq!(config.every_seconds("network"), 10);
    }

    #[test]
    fn a_period_for_a_collector_this_build_has_never_heard_of_is_refused_by_name() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("schedule-typo.yaml");
        std::fs::write(&path, "schedule:\n  proccesses: 60\n").expect("write");

        let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

        assert!(error.cause.contains("proccesses"), "{error}");
        assert!(error.cause.contains("processes"), "{error}");
    }

    #[test]
    fn a_period_for_a_collector_that_is_switched_off_is_refused_by_name() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("schedule-off.yaml");
        std::fs::write(&path, "collectors: [network]\nschedule:\n  users: 60\n").expect("write");

        let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

        assert!(error.cause.contains("users"), "{error}");
        assert!(error.cause.contains("does not run"), "{error}");
    }

    #[test]
    fn a_period_of_zero_seconds_is_refused_rather_than_read_without_a_pause() {
        let dir = std::env::temp_dir().join("vigil-config-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("schedule-zero.yaml");
        std::fs::write(&path, "schedule:\n  network: 0\n").expect("write");

        let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

        assert!(error.cause.contains("no pause"), "{error}");

        std::fs::write(&path, "interval_seconds: 0\n").expect("write");
        let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");
        assert!(error.cause.contains("no pause"), "{error}");
    }
}
