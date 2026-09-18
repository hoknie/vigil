use crate::Config;
use crate::config::{Followed, Stamp};
use crate::loops::Watch;

pub fn of(
    path: &str,
    stamp: Result<Stamp, String>,
    config: &Config,
    watches: &[Watch],
) -> Followed {
    let watched: Vec<&str> = watches.iter().map(|watch| watch.name()).collect();
    let followed = Followed::of(path, stamp, config, crate::modules::modules(), &watched);

    eprintln!("  configuration: {}", said(&followed.names()));
    followed
}

fn said(names: &[&str]) -> String {
    let mut again = vec!["suppressions"];
    again.extend(names);
    format!(
        "{} taken from the files again on the round after they change; everything else, \
         the console switches among it, is read at start-up only",
        again.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_this_daemon_takes_up_without_a_restart_is_said_at_start_up_either_way() {
        let following = said(&["files"]);

        assert!(
            following.contains("files") && following.contains("start-up only"),
            "an operator who edits the file has to be told in the start-up lines which edits \
             reach a running daemon and which wait for a restart: {following}"
        );
        assert!(
            said(&[]).contains("start-up only"),
            "a host that watches no files still reads the rest of its file once"
        );
        assert!(
            said(&[]).contains("suppressions"),
            "and takes up what it is told to silence without a restart"
        );
    }
}
