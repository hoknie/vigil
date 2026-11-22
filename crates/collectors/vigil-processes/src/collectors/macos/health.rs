use vigil_collect::{Health, Limit, Standing, named_account, processes_running, this_account};

pub(super) fn standing() -> Standing {
    let entries = match processes_running() {
        Ok(entries) => entries,
        Err(error) => return Standing::of(Health::Unavailable(error.to_string())),
    };

    let me = this_account();
    let others = entries.iter().any(|entry| entry.real_uid != me);

    Standing::limited(Health::Ok, shown_to(me, others))
}

pub(super) fn shown_to(me: u32, others: bool) -> Option<Limit> {
    if me == 0 || !others {
        return None;
    }

    Limit::of(vec![format!(
        "the command lines of the processes of every account other than {} are not read, \
         and a program of theirs whose file was deleted is not named: macOS shows the arguments \
         of a process only to its own account and to root; run as root",
        named_account(me)
    )])
}
