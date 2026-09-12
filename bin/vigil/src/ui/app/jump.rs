use super::App;

use crate::ui::screens::ports;
use crate::ui::{Anchor, Gone, Level, Origin, Program, Screen, Startup, Subject};

impl App {
    pub(super) fn jump_to_object(&mut self) {
        if self.nav.at() != Screen::Findings {
            return;
        }
        let Some(finding) = self.selected_finding() else {
            return;
        };
        let from = Origin::new(Screen::Findings, finding.event_id.clone());
        let Some(anchor) = Anchor::of(finding) else {
            self.message = Some(
                "This console has no section for the kind of object that finding is about."
                    .to_string(),
            );
            return;
        };

        let last_seen = Gone::of(finding, anchor.key.clone());

        if let Some(collector) = self.reading_needed(&anchor) {
            self.fetch_reading(collector);
        }

        let gone = match self.point_at(&anchor) {
            true => None,
            false => match anchor.screen {
                Screen::Firewall | Screen::Summary => Some(last_seen),
                _ => {
                    self.message = Some(format!(
                        "{} is not in the reading on the {} screen: gone since.",
                        anchor.key,
                        anchor.screen.name()
                    ));
                    return;
                }
            },
        };

        self.gone = gone;
        self.nav.jump(anchor.screen, from);
        self.detail_open = false;
        self.level = Level::List;
        self.refresh_wanted = true;
    }

    fn point_at(&mut self, anchor: &Anchor) -> bool {
        match anchor.screen {
            Screen::Ports => {
                let keys = self.ports_keys();
                self.nav
                    .lists
                    .ports
                    .cursor_mut()
                    .point_at(&anchor.key, &keys)
                    || {
                        self.nav.lists.ports.show(ports::Arrangement::default());
                        let keys = self.ports_keys();
                        self.nav
                            .lists
                            .ports
                            .cursor_mut()
                            .point_at(&anchor.key, &keys)
                    }
            }
            Screen::Accounts => {
                self.nav.lists.accounts.show(Subject::holding(&anchor.key));
                let keys = self.accounts_keys();
                self.nav
                    .lists
                    .accounts
                    .cursor_mut()
                    .point_at(&anchor.key, &keys)
            }
            Screen::Programs => {
                self.nav.lists.programs.show(Program::holding(&anchor.key));
                let keys = self.programs_keys();
                self.nav
                    .lists
                    .programs
                    .cursor_mut()
                    .point_at(&anchor.key, &keys)
            }
            Screen::Startup => {
                self.nav.lists.startup.show(Startup::holding(&anchor.key));
                let keys = self.startup_keys();
                self.nav
                    .lists
                    .startup
                    .cursor_mut()
                    .point_at(&anchor.key, &keys)
            }
            Screen::System => {
                self.nav
                    .lists
                    .system
                    .show(crate::ui::System::holding(&anchor.key));
                let keys = self.system_keys();
                self.nav
                    .lists
                    .system
                    .cursor_mut()
                    .point_at(&anchor.key, &keys)
            }
            Screen::Firewall | Screen::Containers => {
                let screen = anchor.screen;
                let keys = match screen {
                    Screen::Containers => {
                        crate::ui::screens::containers::identities(&self.view, &self.contained())
                    }
                    other => self.rows_of(other),
                };
                let key = anchor.key.clone();
                self.one_mut(screen)
                    .is_some_and(|one| one.cursor_mut().point_at(&key, &keys))
            }
            Screen::Summary => self.view.status.as_ref().is_some_and(|status| {
                status.agent.buffers.as_ref().is_some_and(|buffers| {
                    buffers.iter().any(|buffer| buffer.receiver == anchor.key)
                })
            }),
            _ => false,
        }
    }
}
