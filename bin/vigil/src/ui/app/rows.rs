use vigil_model::Finding;

use super::App;

use crate::ui::Arrows;
use crate::ui::screens::{accounts, ports, programs, startup};

impl App {
    pub(super) fn showing(&self) -> accounts::Showing<'_> {
        accounts::Showing {
            subject: self.nav.lists.accounts.showing(),
            search: self.nav.lists.accounts.search(),
            cursor: self.nav.lists.accounts.at(),
            elsewhere: self.nav.lists.accounts.narrowed_elsewhere(),
            arrows: Arrows::at(self.level),
        }
    }

    pub(super) fn listening(&self) -> ports::Showing<'_> {
        ports::Showing {
            arrangement: self.nav.lists.ports.showing(),
            protocols: &self.ports_protocols,
            search: self.nav.lists.ports.search(),
            cursor: self.nav.lists.ports.at(),
            arrows: Arrows::at(self.level),
        }
    }

    pub(super) fn running(&self) -> programs::Showing<'_> {
        programs::Showing {
            program: self.nav.lists.programs.showing(),
            search: self.nav.lists.programs.search(),
            cursor: self.nav.lists.programs.at(),
            elsewhere: self.nav.lists.programs.narrowed_elsewhere(),
            arrows: Arrows::at(self.level),
        }
    }

    pub(super) fn starting(&self) -> startup::Showing<'_> {
        startup::Showing {
            list: self.nav.lists.startup.showing(),
            search: self.nav.lists.startup.search(),
            cursor: self.nav.lists.startup.at(),
            elsewhere: self.nav.lists.startup.narrowed_elsewhere(),
            arrows: Arrows::at(self.level),
        }
    }

    pub(super) fn ports_rows(&self) -> Vec<ports::Row<'_>> {
        ports::rows(&self.view, &self.listening())
    }

    pub(super) fn ports_keys(&self) -> Vec<String> {
        ports::keys(&self.view, &self.listening())
    }

    pub(super) fn accounts_rows(&self) -> Vec<accounts::Row<'_>> {
        accounts::rows(
            &self.view,
            self.nav.lists.accounts.showing(),
            self.nav.lists.accounts.search(),
        )
    }

    pub(super) fn accounts_keys(&self) -> Vec<String> {
        accounts::keys(
            &self.view,
            self.nav.lists.accounts.showing(),
            self.nav.lists.accounts.search(),
        )
    }

    pub(super) fn programs_rows(&self) -> Vec<programs::Row<'_>> {
        programs::rows(
            &self.view,
            self.nav.lists.programs.showing(),
            self.nav.lists.programs.search(),
        )
    }

    pub(super) fn programs_keys(&self) -> Vec<String> {
        programs::keys(
            &self.view,
            self.nav.lists.programs.showing(),
            self.nav.lists.programs.search(),
        )
    }

    pub(super) fn startup_rows(&self) -> Vec<startup::Row<'_>> {
        startup::rows(
            &self.view,
            self.nav.lists.startup.showing(),
            self.nav.lists.startup.search(),
        )
    }

    pub(super) fn startup_keys(&self) -> Vec<String> {
        startup::keys(
            &self.view,
            self.nav.lists.startup.showing(),
            self.nav.lists.startup.search(),
        )
    }

    pub(super) fn selected_finding(&self) -> Option<&Finding> {
        self.filter
            .passing(&self.view.found.findings)
            .get(self.nav.findings.at())
            .copied()
    }
}
