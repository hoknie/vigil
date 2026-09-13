use vigil_model::{Golden, Settled};

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
