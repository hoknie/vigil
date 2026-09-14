use vigil_model::Finding;

use super::App;

use crate::ui::screens::findings;

impl App {
    pub(super) fn found(&self) -> findings::Showing<'_> {
        findings::Showing {
            filter: &self.filter,
            cursor: self.nav.findings.at(),
            focused: self.level == crate::ui::Level::List,
            sorting: self.sorted(),
            picked: &self.picked,
            dismissed: &self.dismissed,
        }
    }

    pub(super) fn passing(&self) -> Vec<&Finding> {
        let mut passing = self
            .dismissed
            .keeping(self.filter.passing(&self.view.found.findings));
        findings::sort(&mut passing, self.sorted());
        passing
    }

    pub(super) fn selected_finding(&self) -> Option<&Finding> {
        self.passing().get(self.nav.findings.at()).copied()
    }
}
