pub const CONFIGURE_LIVES_IN_THE_DAEMON: &str = "\
vigil: `configure` lives in the daemon:

    vigild configure [PATH] [-f] [-d]

Writing a configuration asks every collector what it can watch here; the console
links no collectors. Both binaries ship in one package.";

pub const COLLECTOR_LIVES_IN_THE_DAEMON: &str = "\
vigil: switching a collector on or off lives in the daemon:

    vigild collector <name> enable [--config PATH] [-d]
    vigild collector <name> disable [--config PATH] [-d]

It writes /etc/vigil/vigil.yaml and starts what has to run on this host for that
reading, so it is the daemon's to do. Both binaries ship in one package.";

pub const UI_MOVED: &str = "\
vigil: opening the console is `vigil ui` now:

    vigil ui [--socket PATH] [--screen NAME]

The flags are unchanged.";

pub const CAPTURE_MOVED: &str = "\
vigil: printing one screen as text is `vigil capture` now:

    vigil capture [--socket PATH] [--screen NAME]

The same page and the same flags. Exit code: 0 the agent answered, 1 it did not.";

pub fn the_old_shape(arguments: &[String]) -> Option<&'static str> {
    let first = arguments.get(1)?;
    let word = first.split('=').next().unwrap_or(first);

    match word {
        "--once" => Some(CAPTURE_MOVED),
        "--socket" | "--screen" => match arguments
            .iter()
            .any(|word| word.split('=').next().unwrap_or(word) == "--once")
        {
            true => Some(CAPTURE_MOVED),
            false => Some(UI_MOVED),
        },
        _ => None,
    }
}
