use crate::Section;

pub fn a_section_gives_every_pane_a_group_or_none_of_them(section: &dyn Section) {
    let panes = section.panes();
    let named = section.groups();
    let carried = panes
        .iter()
        .filter(|pane| pane.belongs_to().is_some())
        .count();

    if carried == 0 {
        assert!(
            named.is_empty(),
            "the section {} names the groups {named:?} and no pane of it belongs to one: the \
             first row would be drawn over a second row that never changes, and every name on \
             it would be a key that does nothing",
            section.name()
        );
        return;
    }

    assert_eq!(
        carried,
        panes.len(),
        "the section {} gives a group to {carried} of its {} lists: a list belonging to no \
         group is a list no row of the menu reaches, and the reader meets it only by the \
         cursor landing on it from somewhere else",
        section.name(),
        panes.len()
    );
    assert!(
        !named.is_empty(),
        "the lists of the section {} belong to groups and the section names none: the order \
         of the first row is the section's to say, and without it the row is drawn in the \
         order the panes happen to be declared in",
        section.name()
    );
    for pane in &panes {
        let group = pane.belongs_to().unwrap_or_default();
        assert!(
            named.contains(&group),
            "the list {} of the section {} belongs to {group:?}, which the section does not \
             name: the first row is drawn from what the section names, so this list is drawn \
             under no group at all",
            pane.name(),
            section.name()
        );
    }
}

pub fn every_group_the_section_names_is_carried_by_a_pane(section: &dyn Section) {
    let panes = section.panes();

    for group in section.groups() {
        assert!(
            panes.iter().any(|pane| pane.belongs_to() == Some(group)),
            "the section {} names the group {group:?} and no list of it belongs there: a name \
             on the first row with nothing behind it is a name that answers a keypress with an \
             empty screen, where a group whose lists have nothing to show says so in a \
             sentence",
            section.name()
        );
    }
}

pub fn run_all_of_the_section(section: &dyn Section) {
    a_section_gives_every_pane_a_group_or_none_of_them(section);
    every_group_the_section_names_is_carried_by_a_pane(section);
}
