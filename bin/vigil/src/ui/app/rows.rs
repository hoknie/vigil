use vigil_model::Finding;

use super::App;

use crate::ui::Arrows;
use crate::ui::details::reading::Subject;
use crate::ui::screens::{
    accounts, containers, findings, firewall, ports, programs, startup, system,
};
use crate::ui::{One, Screen, System};

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
            sorting: self.sorted(),
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
            nesting: self.startup_nesting,
            cursor: self.nav.lists.startup.at(),
            elsewhere: self.nav.lists.startup.narrowed_elsewhere(),
            arrows: Arrows::at(self.level),
        }
    }

    pub(super) fn one(&self, screen: Screen) -> Option<&One> {
        match screen {
            Screen::Firewall => Some(&self.nav.lists.firewall),
            Screen::Containers => Some(&self.nav.lists.containers),
            _ => None,
        }
    }

    pub(super) fn one_mut(&mut self, screen: Screen) -> Option<&mut One> {
        match screen {
            Screen::Firewall => Some(&mut self.nav.lists.firewall),
            Screen::Containers => Some(&mut self.nav.lists.containers),
            _ => None,
        }
    }

    pub(super) fn filtering(&self) -> firewall::Showing<'_> {
        firewall::Showing {
            search: self.nav.lists.firewall.search(),
            cursor: self.nav.lists.firewall.at(),
            arrows: Arrows::at(self.level),
            gone: self.gone.as_ref(),
            sorting: self.sorted(),
        }
    }

    pub(super) fn firewall_rows(&self) -> Vec<firewall::Row<'_>> {
        firewall::rows(&self.view, &self.filtering())
    }

    pub(super) fn firewall_keys(&self) -> Vec<String> {
        self.firewall_rows()
            .into_iter()
            .map(|row| row.key)
            .collect()
    }

    pub(super) fn made_of(&self) -> system::Showing<'_> {
        system::Showing {
            showing: self.nav.lists.system.showing(),
            search: self.nav.lists.system.search(),
            cursor: self.nav.lists.system.at(),
            elsewhere: self.nav.lists.system.narrowed_elsewhere(),
            arrows: Arrows::at(self.level),
            gone: self.gone.as_ref(),
            sorting: self.sorted(),
        }
    }

    pub(super) fn system_keys(&self) -> Vec<String> {
        let showing = self.made_of();
        match showing.showing {
            System::Host => system::host::rows(&self.view, &showing)
                .into_iter()
                .map(|row| row.key)
                .collect(),
            System::Files => system::files::rows(&self.view, &showing)
                .into_iter()
                .map(|row| row.key)
                .collect(),
        }
    }

    pub(super) fn contained(&self) -> containers::Showing<'_> {
        containers::Showing {
            search: self.nav.lists.containers.search(),
            cursor: self.nav.lists.containers.at(),
            arrows: Arrows::at(self.level),
            gone: self.gone.as_ref(),
            sorting: self.sorted(),
        }
    }

    pub(super) fn containers_rows(&self) -> Vec<containers::Row<'_>> {
        containers::rows(&self.view, &self.contained())
    }

    pub(super) fn containers_keys(&self) -> Vec<String> {
        self.containers_rows()
            .into_iter()
            .map(|row| row.key)
            .collect()
    }

    pub(super) fn reading_subject(&self) -> Option<Subject<'_>> {
        match self.nav.at() {
            Screen::System => match self.nav.lists.system.showing() {
                System::Host => {
                    let showing = self.made_of();
                    let rows = system::host::rows(&self.view, &showing);
                    let row = rows.get(showing.cursor)?;
                    Some(Subject {
                        key: row.key.clone(),
                        kind: row.kind.name(),
                        named: system::host::fields::what(row),
                        means: system::host::fields::means(row),
                        item: row.item,
                    })
                }
                System::Files => {
                    let showing = self.made_of();
                    let rows = system::files::rows(&self.view, &showing);
                    let row = rows.get(showing.cursor)?;
                    Some(Subject {
                        key: row.key.clone(),
                        kind: row.kind.name(),
                        named: system::files::fields::what(row),
                        means: system::files::fields::means(row),
                        item: row.item,
                    })
                }
            },
            Screen::Containers => {
                let rows = self.containers_rows();
                let row = rows.get(self.nav.lists.containers.at())?;
                Some(Subject {
                    key: row.key.clone(),
                    kind: row.kind.name(),
                    named: containers::fields::what(row),
                    means: containers::fields::means(row),
                    item: row.item,
                })
            }
            _ => None,
        }
    }

    pub(super) fn ports_rows(&self) -> Vec<ports::Row<'_>> {
        ports::rows(&self.view, &self.listening())
    }

    pub(super) fn ports_keys(&self) -> Vec<String> {
        self.ports_rows().into_iter().map(|row| row.key).collect()
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
        startup::rows(&self.view, &self.starting())
    }

    pub(super) fn startup_keys(&self) -> Vec<String> {
        startup::keys(&self.view, &self.starting())
    }

    pub(super) fn found(&self) -> findings::Showing<'_> {
        findings::Showing {
            filter: &self.filter,
            cursor: self.nav.findings.at(),
            focused: self.level == crate::ui::Level::List,
            sorting: self.sorted(),
        }
    }

    pub(super) fn passing(&self) -> Vec<&Finding> {
        let mut passing = self.filter.passing(&self.view.found.findings);
        findings::sort(&mut passing, self.sorted());
        passing
    }

    pub(super) fn selected_finding(&self) -> Option<&Finding> {
        self.passing().get(self.nav.findings.at()).copied()
    }
}
