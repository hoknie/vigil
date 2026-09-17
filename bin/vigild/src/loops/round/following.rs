use std::time::Instant;

use crate::helpers::rfc3339;
use crate::types::Said;

use super::Round;

impl Round {
    pub(super) fn follow_the_file(&mut self, said: &mut Said) {
        let looked = self.followed.look();
        for line in &looked.said {
            eprintln!("{} {line}", rfc3339::now());
        }

        for (name, settings) in looked.refollowed {
            let (Some(index), Some(module)) =
                (self.schedule.index_of(name), self.followed.module(name))
            else {
                continue;
            };

            match module.collector(&settings) {
                Err(why) => eprintln!(
                    "{} collector {name}: what the configuration names now could not be taken \
                     up, so what it named before is still watched: {why}",
                    rfc3339::now()
                ),
                Ok(collector) => {
                    let rules = module.rules(&settings);
                    self.watches[index].follow(collector, rules);
                    eprintln!(
                        "{} collector {name}: the configuration changed, and what it names now \
                         is read on this round",
                        rfc3339::now()
                    );
                    self.read(index, said);
                    self.schedule.restart(index, Instant::now());
                }
            }
        }
    }
}
