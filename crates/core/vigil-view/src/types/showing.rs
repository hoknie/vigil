use super::sorting::Sorting;

#[derive(Debug, Clone, Copy, Default)]
pub struct Showing<'a> {
    pub search: &'a str,
    pub chosen: &'a [&'a str],
    pub sorting: Sorting,
}

impl<'a> Showing<'a> {
    pub fn searching(search: &'a str) -> Showing<'a> {
        Showing {
            search,
            ..Showing::default()
        }
    }

    pub fn showing(self, chosen: &'a [&'a str]) -> Showing<'a> {
        Showing { chosen, ..self }
    }

    pub fn sorted(self, sorting: Sorting) -> Showing<'a> {
        Showing { sorting, ..self }
    }

    pub fn wants(&self, toggle: &str) -> bool {
        self.chosen.is_empty() || self.chosen.contains(&toggle)
    }
}
