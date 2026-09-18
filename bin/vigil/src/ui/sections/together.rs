use vigil_view::{Pane, Section};

type Declared = Vec<Box<dyn Section>>;

pub(super) fn together(declared: Declared) -> Declared {
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

    fn groups(&self) -> Vec<&'static str> {
        let mut named: Vec<&'static str> = Vec::new();
        for group in self.0.iter().flat_map(|section| section.groups()) {
            if !named.contains(&group) {
                named.push(group);
            }
        }
        named
    }

    fn shows_what_has_gone(&self) -> bool {
        self.0.iter().any(|section| section.shows_what_has_gone())
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::Snapshot;
    use vigil_view::{Cell, Column, Notice, Piece, Room, RowKey, Showing, Width};

    use super::*;

    struct Grouped(&'static str, &'static [&'static str]);

    struct In(&'static str);

    impl Section for Grouped {
        fn name(&self) -> &'static str {
            "containers"
        }
        fn title(&self) -> &'static str {
            self.0
        }
        fn holds(&self) -> &'static str {
            self.0
        }
        fn panes(&self) -> Vec<Box<dyn Pane>> {
            self.1
                .iter()
                .map(|group| Box::new(In(group)) as Box<dyn Pane>)
                .collect()
        }
        fn groups(&self) -> Vec<&'static str> {
            let mut named = self.1.to_vec();
            named.dedup();
            named
        }
    }

    impl Pane for In {
        fn name(&self) -> &str {
            self.0
        }
        fn belongs_to(&self) -> Option<&'static str> {
            Some(self.0)
        }
        fn caption(&self) -> &str {
            self.0
        }
        fn about(&self) -> &str {
            self.0
        }
        fn reads(&self) -> &str {
            self.0
        }
        fn columns(&self, _room: Room) -> Vec<Column> {
            vec![Column::new("NAME", Width::Share(1))]
        }
        fn rows(&self, _reading: &Snapshot, _showing: &Showing<'_>) -> Vec<RowKey> {
            Vec::new()
        }
        fn cells(&self, _reading: &Snapshot, _row: &RowKey, _room: Room) -> Vec<Cell> {
            Vec::new()
        }
        fn detail(&self, _reading: &Snapshot, _row: &RowKey, _width: usize) -> Vec<Piece> {
            Vec::new()
        }
        fn tally(&self, _reading: &Snapshot, _showing: &Showing<'_>, _shown: usize) -> String {
            String::new()
        }
        fn empty(&self, _showing: &Showing<'_>) -> Notice {
            Notice::plain(self.0)
        }
    }

    #[test]
    fn two_modules_drawing_one_grouped_section_give_it_their_groups_in_the_order_they_are_linked() {
        let merged = together(vec![
            Box::new(Grouped("what runs in containers", &["host"])),
            Box::new(Grouped(
                "what the engines hold",
                &["docker", "podman", "podman"],
            )),
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(
            merged[0].groups(),
            vec!["host", "docker", "podman"],
            "the first row of the menu is drawn from the groups the section names, so a merge \
             that keeps the groups of its first half only is a screen whose second half no \
             key reaches"
        );
        assert_eq!(merged[0].panes().len(), 4);
        assert_eq!(merged[0].title(), "what runs in containers");
        vigil_view::conformance::run_all_of_the_section(merged[0].as_ref());
    }
}
