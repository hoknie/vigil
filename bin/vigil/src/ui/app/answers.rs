use vigil_model::{Request, Response};

use super::App;
use crate::link::{Trouble, TroubleKind};

use crate::ui::{Anchor, Program, Reading, Refusal, Screen, Status, View};

impl App {
    pub(super) fn wanted_reading(&self) -> Option<&'static str> {
        let collector = match self.nav.at() {
            Screen::Ports => "ports",
            Screen::Accounts => "users",
            Screen::Programs => self.nav.lists.programs.showing().collector(),
            Screen::Startup => crate::ui::screens::startup::COLLECTOR,
            Screen::Firewall => crate::ui::screens::firewall::COLLECTOR,
            Screen::Home | Screen::Summary | Screen::Findings => return None,
        };
        match self.view.switched_off(collector) {
            true => None,
            false => Some(collector),
        }
    }

    pub(super) fn reading_needed(&self, anchor: &Anchor) -> Option<&'static str> {
        let collector = match anchor.screen {
            Screen::Ports => "ports",
            Screen::Accounts => "users",
            Screen::Programs => Program::holding(&anchor.key).collector(),
            Screen::Startup => crate::ui::screens::startup::COLLECTOR,
            Screen::Firewall => crate::ui::screens::firewall::COLLECTOR,
            Screen::Home | Screen::Summary | Screen::Findings => return None,
        };
        match self.view.reading(collector) {
            Reading::Unknown => Some(collector),
            _ => None,
        }
    }

    pub(super) fn fetch_reading(&mut self, collector: &str) {
        let requests = vec![Request::Snapshot {
            collector: collector.to_string(),
        }];

        let answers = match self.link.ask(&requests) {
            Ok(answers) => answers,
            Err(trouble) => {
                self.view.trouble = Some(trouble);
                return;
            }
        };

        let mut view = std::mem::replace(&mut self.view, View::nothing_yet(self.link.path()));
        for (request, answer) in requests.iter().zip(answers) {
            self.read(&mut view, request, answer);
        }
        self.view = view;
    }

    pub fn refresh(&mut self) {
        self.refresh_wanted = false;

        let mut requests = vec![Request::Status, Request::Findings { limit: None }];
        if let Some(collector) = self.wanted_reading() {
            requests.push(Request::Snapshot {
                collector: collector.to_string(),
            });
        }

        let answers = match self.link.ask(&requests) {
            Ok(answers) => answers,
            Err(trouble) => {
                self.view.trouble = Some(trouble);
                return;
            }
        };

        let mut view = View::nothing_yet(self.link.path());
        for (request, answer) in requests.iter().zip(answers) {
            self.read(&mut view, request, answer);
        }

        self.view = view;
        self.settle();
    }

    pub(super) fn read(&self, view: &mut View, request: &Request, answer: Response) {
        match answer {
            Response::Status {
                host,
                agent,
                sent_at,
                ..
            } => {
                view.status = Some(Status {
                    host: *host,
                    agent: *agent,
                    sent_at,
                })
            }
            Response::Snapshot {
                collector,
                snapshot,
                refusal,
            } => view.readings.put(
                collector,
                match (snapshot, refusal) {
                    (Some(snapshot), _) => Reading::Taken(snapshot),
                    (None, Some(refusal)) => Reading::Refused(Refusal::told(refusal)),
                    (None, None) => Reading::NotTakenYet,
                },
            ),
            Response::Findings {
                findings,
                dropped,
                capacity,
            } => {
                view.found.findings = findings;
                view.found.dropped = dropped;
                view.found.capacity = capacity;
            }
            Response::Error { error } => match request {
                Request::Snapshot { collector } => view.readings.put(
                    collector.clone(),
                    Reading::Refused(Refusal::answered(error.message)),
                ),
                Request::Findings { .. } => view.found.refused = Some(error.message),
                Request::Status => {
                    view.trouble = Some(Trouble::new(
                        self.link.path(),
                        TroubleKind::Unreadable,
                        error.message,
                    ))
                }
            },
        }
    }

    pub(super) fn running_list(&self) -> Program {
        self.nav.lists.programs.showing()
    }
}
