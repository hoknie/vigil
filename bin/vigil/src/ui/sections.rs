#[cfg(test)]
pub(crate) mod instead;
mod together;

use vigil_module::Module;
use vigil_view::Section;

use together::together;

pub fn modules() -> Vec<Box<dyn Module>> {
    vec![
        Box::new(vigil_network::Network),
        Box::new(vigil_users::Users),
        Box::new(vigil_processes::Processes),
        Box::new(vigil_launches::Launches),
        Box::new(vigil_persistence::Persistence),
        Box::new(vigil_firewall::Firewall),
        Box::new(vigil_resources::Resources),
        Box::new(vigil_files::Files),
        Box::new(vigil_containers::Containers),
        Box::new(vigil_engines::Engines),
    ]
}

pub fn sections() -> Declared {
    let declared = together(
        modules()
            .iter()
            .filter_map(|module| module.section())
            .collect(),
    );

    #[cfg(test)]
    let declared = declared
        .into_iter()
        .map(|section| instead::of(section.name()).unwrap_or(section))
        .collect();

    declared
}

pub fn holding(name: &str) -> Option<Box<dyn Section>> {
    sections()
        .into_iter()
        .find(|section| section.name() == name)
}

type Declared = Vec<Box<dyn Section>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_section_this_console_draws_groups_all_of_its_lists_or_none_of_them() {
        for section in sections() {
            vigil_view::conformance::run_all_of_the_section(section.as_ref());
        }
    }

    #[test]
    fn a_section_put_in_the_place_of_another_stands_only_while_the_test_holds_it() {
        let named = crate::ui::fixture::grouped::SECTION;
        let reads = |section: Box<dyn Section>| -> Vec<String> {
            let mut read: Vec<String> = section
                .panes()
                .iter()
                .map(|pane| pane.reads().to_string())
                .collect();
            read.dedup();
            read
        };
        {
            let _standing = instead::drawn(named, crate::ui::fixture::grouped::section);
            assert_eq!(
                reads(holding(named).expect("a section under that name")),
                vec!["containers"]
            );
        }

        assert_eq!(
            reads(holding(named).expect("a section under that name")),
            vec!["containers", "containers-engines"],
            "a section put in the place of another for one test is put back when the test \
             ends, or the next test reads a console this build does not ship"
        );
    }

    #[test]
    fn the_containers_screen_is_the_host_then_every_engine_as_the_modules_are_linked() {
        let containers = holding("containers").expect("the containers screen");

        assert_eq!(
            containers.groups(),
            vec!["host", "docker", "podman"],
            "what /proc sees comes first, because it sees a container of any runtime, and the \
             engines follow in the order their dumps are read"
        );
        assert_eq!(
            containers
                .panes()
                .first()
                .map(|pane| pane.reads().to_string()),
            Some("containers".to_string())
        );
    }

    #[test]
    fn every_screen_a_module_declares_is_a_screen_this_console_draws() {
        let mut declared = 0;

        for module in modules() {
            let Some(section) = module.section() else {
                continue;
            };
            declared += 1;
            assert!(
                sections()
                    .iter()
                    .any(|drawn| drawn.name() == section.name()),
                "{} declares the section {} and the console draws no such thing",
                module.name(),
                section.name()
            );
        }

        assert!(
            declared > 0,
            "no module declares a screen, so this guard reads nothing"
        );
    }

    #[test]
    fn a_module_with_no_screen_of_its_own_is_opened_as_the_plain_list_of_what_it_read() {
        let without: Vec<&'static str> = modules()
            .iter()
            .filter(|module| module.section().is_none())
            .map(|module| module.name())
            .collect();

        for name in &without {
            assert!(
                crate::ui::Screen::showing(name).is_none(),
                "{name} is drawn by a screen and declares no section, so nothing decides \
                 which of the two the reader gets"
            );
        }
        assert_eq!(
            without,
            Vec::<&str>::new(),
            "a reading no screen of this build draws is shown as a plain list rather than \
             hidden, and which readings those are is a thing this console states out loud: \
             this build has none"
        );
    }

    #[test]
    fn two_modules_reading_two_things_about_one_subject_are_one_section_with_two_lists() {
        let programs = holding("programs").expect("what has run here");

        assert_eq!(
            programs
                .panes()
                .iter()
                .map(|pane| pane.reads())
                .collect::<Vec<_>>(),
            vec!["processes", "launches"],
            "what runs now and what was run are one screen: a reader looking for a program \
             should not have to know which of the two collectors saw it"
        );
    }

    #[test]
    fn every_section_is_named_once_however_many_modules_write_to_it() {
        let mut named: Vec<&str> = sections().iter().map(|section| section.name()).collect();
        let total = named.len();
        named.sort_unstable();
        named.dedup();

        assert_eq!(
            named.len(),
            total,
            "a section drawn twice is a section a number opens at random"
        );
    }

    #[test]
    fn the_modules_of_one_section_agree_on_what_to_write_over_it() {
        let programs = holding("programs").expect("what has run here");

        assert_eq!(programs.title(), "What has run here");
        assert_eq!(programs.holds(), "what has run here");
    }
}
