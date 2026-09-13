use crate::{COLLECTORS, Collector};

#[test]
fn every_collector_this_build_has_is_in_the_product_vocabulary_and_the_other_way_round() {
    let now = || "2026-09-09T12:00:00.000Z".to_string();
    let built: Vec<&'static str> = vec![
        super::PersistenceCollector::new(now).name(),
        super::ProcessesCollector::new(now).name(),
        super::LaunchesCollector::new(now, false).name(),
        super::ResourcesCollector::new(now).name(),
        super::FilesCollector::new(now, &[], 0).name(),
    ];

    for name in &built {
        assert!(
            crate::is_known_collector(name),
            "this build has a collector called {name} that the vocabulary does not list"
        );
    }
    for collector in COLLECTORS {
        assert!(
            built.contains(&collector.name),
            "the vocabulary lists {} and this build has no such collector",
            collector.name
        );
    }
}
