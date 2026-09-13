use super::App;

use crate::ui::{Anchor, Gone, Level, Origin, Program, Screen, Startup, holding};

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
            screen if holding(screen.name()).is_some() => {
                let Some(row) = self.row_named(screen, &anchor.key) else {
                    return false;
                };
                if let Some(at) = self.pane_of_holding(screen, &row)
                    && let Some(panes) = self.panes_of_mut(screen)
                {
                    panes.show(at);
                }
                let keys = self.pane_keys_of(screen);
                self.panes_of_mut(screen)
                    .is_some_and(|panes| panes.cursor_mut().point_at(&row, &keys))
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
            Screen::Summary => self.view.status.as_ref().is_some_and(|status| {
                status.agent.buffers.as_ref().is_some_and(|buffers| {
                    buffers.iter().any(|buffer| buffer.receiver == anchor.key)
                })
            }),
            _ => false,
        }
    }
}
