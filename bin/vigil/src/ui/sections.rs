use vigil_module::Module;
use vigil_view::{Pane, Section};

pub fn modules() -> Vec<Box<dyn Module>> {
    vec![
        Box::new(vigil_network::Ports),
        Box::new(vigil_users::Users),
        Box::new(vigil_processes::Processes),
        Box::new(vigil_launches::Launches),
        Box::new(vigil_persistence::Persistence),
        Box::new(vigil_firewall::Firewall),
        Box::new(vigil_resources::Resources),
        Box::new(vigil_files::Files),
        Box::new(vigil_containers::Containers),
    ]
}

pub fn sections() -> Declared {
    together(
        modules()
            .iter()
            .filter_map(|module| module.section())
            .collect(),
    )
}

pub fn holding(name: &str) -> Option<Box<dyn Section>> {
    sections()
        .into_iter()
        .find(|section| section.name() == name)
}

type Declared = Vec<Box<dyn Section>>;

fn together(declared: Declared) -> Declared {
    let mut named: Vec<&'static str> = Vec::new();
    for section in &declared {
        if !named.contains(&section.name()) {
            named.push(section.name());
        }
    }

    let mut sections: Declared = Vec::new();
    let mut left = declared;
    for name in named {
        let (mine, rest): (Declared, Declared) =
            left.into_iter().partition(|section| section.name() == name);
        left = rest;
        sections.push(match mine.len() {
            1 => mine.into_iter().next().expect("one section"),
            _ => Box::new(Together(mine)),
        });
    }

    sections
}

struct Together(Vec<Box<dyn Section>>);

impl Section for Together {
    fn name(&self) -> &'static str {
        self.0[0].name()
    }

    fn title(&self) -> &'static str {
        self.0[0].title()
    }

    fn holds(&self) -> &'static str {
        self.0[0].holds()
    }

    fn panes(&self) -> Vec<Box<dyn Pane>> {
        self.0.iter().flat_map(|section| section.panes()).collect()
    }

    fn shows_what_has_gone(&self) -> bool {
        self.0.iter().any(|section| section.shows_what_has_gone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_module_linked_into_this_console_declares_the_screen_that_draws_its_reading() {
        for module in modules() {
            let section = module.section().unwrap_or_else(|| {
                panic!(
                    "{} is linked into this console and declares no section: its reading \
                     would arrive with nowhere to be drawn",
                    module.name()
                )
            });
            assert!(
                sections()
                    .iter()
                    .any(|drawn| drawn.name() == section.name()),
                "{} declares the section {} and the console draws no such thing",
                module.name(),
                section.name()
            );
        }
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
