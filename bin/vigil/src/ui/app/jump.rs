use super::App;

use crate::ui::screens::ports;
use crate::ui::{Anchor, Level, Origin, Program, Screen, Startup, Subject};

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

        if let Some(collector) = self.reading_needed(&anchor) {
            self.fetch_reading(collector);
        }

        let found = self.point_at(&anchor);
        match found {
            true => {
                self.nav.jump(anchor.screen, from);
                self.detail_open = false;
                self.level = Level::List;
                self.refresh_wanted = true;
            }
            false => {
                self.message = Some(format!(
                    "{} is not in the reading on the {} screen: gone since.",
                    anchor.key,
                    anchor.screen.name()
                ))
            }
        }
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
            _ => false,
        }
    }
}
