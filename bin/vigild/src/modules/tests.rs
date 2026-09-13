use vigil_model::{Golden, Settled};
use vigil_module::Settings;

use super::{every_seconds_of, is_known, modules, names, subject_of, unit_of, watched};

fn declared() -> Settled {
    let mut pinned = Settled::new("collectors");

    for watched in watched() {
        pinned = pinned.pinning(
            watched.name,
            "every_seconds",
            watched.every_seconds,
            format!(
                "the period {} is declared with among the collectors this build ships",
                watched.name
            ),
        );
    }

    pinned
}

#[test]
fn the_period_a_collector_declares_is_published_for_the_screens_that_show_it() {
    if let Err(complaint) = Golden::settled("collectors").write_or_check(&declared().written()) {
        panic!("{complaint}");
    }
}

#[test]
fn what_a_module_declares_about_itself_is_what_the_rest_of_this_daemon_reads() {
    for module in modules() {
        assert_eq!(subject_of(module.name()), Some(module.subject()));
        assert_eq!(
            every_seconds_of(module.name()),
            Some(module.every_seconds())
        );
        assert_eq!(unit_of(module.name()), module.unit());
        assert!(is_known(module.name()));
    }
}

#[test]
fn no_collector_is_watched_twice_under_one_name() {
    let mut seen = names();
    let total = seen.len();
    seen.sort_unstable();
    seen.dedup();

    assert_eq!(
        seen.len(),
        total,
        "a module and a collector of the same name would be read twice and reported twice: {:?}",
        names()
    );
}

#[test]
fn every_module_this_build_ships_is_watched_before_a_configuration_file_is_read() {
    let named = names();

    for module in modules() {
        assert!(
            named.contains(&module.name()),
            "{} is linked into this build and nothing would ever ask it for a reading",
            module.name()
        );
    }
}

#[test]
fn no_family_of_findings_is_claimed_by_two_modules() {
    let mut claimed: Vec<(&str, &str)> = Vec::new();

    for module in modules() {
        for family in module.families() {
            if let Some((already, _)) = claimed.iter().find(|(named, _)| named == family) {
                panic!(
                    "{family} is claimed by {already} and by {}: a finding of that family \
                     would be walked to a row in two readings, and the reader would be sent \
                     to whichever of them the list happened to be in",
                    module.name()
                );
            }
            claimed.push((family, module.name()));
        }
    }

    assert!(!claimed.is_empty());
}

#[test]
fn a_finding_is_answered_for_by_the_module_that_raised_it_and_by_no_other() {
    let every: Vec<Box<dyn vigil_module::Module>> = modules();

    for module in &every {
        for family in module.families() {
            let key = format!("{family}|something");

            let answering: Vec<&'static str> = every
                .iter()
                .filter(|other| other.raised(&key))
                .map(|other| other.name())
                .collect();

            assert_eq!(
                answering,
                vec![module.name()],
                "{key} is answered for by {answering:?}"
            );
        }
    }
}

#[test]
fn a_module_that_names_a_key_of_its_own_refuses_a_key_inside_it_that_it_does_not_know() {
    let at_noon = || "2026-09-13T12:00:00.000Z".to_string();
    let mut named = 0;

    for module in modules() {
        let Some(key) = module.settings_key() else {
            continue;
        };
        named += 1;

        assert!(
            module
                .check(&Settings::of(at_noon, key, serde_json::json!({})))
                .is_ok(),
            "{key}: a block with nothing in it is a module left on its own values"
        );
        let refusal = module
            .check(&Settings::of(
                at_noon,
                key,
                serde_json::json!({"nothing-of-the-sort": 1}),
            ))
            .expect_err(&format!(
                "{key}: a key this module never heard of is read as silence, and an operator \
                 who misspelled one is told nothing"
            ));
        assert!(refusal.contains("nothing-of-the-sort"), "{key}: {refusal}");
    }

    assert!(
        named > 0,
        "no module names a key, so this guard reads nothing"
    );
}

#[test]
fn a_module_with_no_key_of_its_own_is_handed_nothing_and_is_content_with_it() {
    let at_noon = || "2026-09-13T12:00:00.000Z".to_string();

    for module in modules() {
        if module.settings_key().is_some() {
            continue;
        }
        assert!(
            module.check(&Settings::plain(at_noon)).is_ok(),
            "{} takes no settings and refuses to start without them",
            module.name()
        );
    }
}
