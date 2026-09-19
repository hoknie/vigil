use vigil_collect::{Health, name_of_user, processes_running, this_account};

pub(super) fn health() -> Health {
    let entries = match processes_running() {
        Ok(entries) => entries,
        Err(error) => return Health::Unavailable(error.to_string()),
    };

    let me = this_account();
    let others = entries.iter().any(|entry| entry.real_uid != me);

    shown_to(me, others)
}

pub(super) fn shown_to(me: u32, others: bool) -> Health {
    if me == 0 || !others {
        return Health::Ok;
    }

    let named = match name_of_user(me) {
        Some(name) => format!("{name} (uid {me})"),
        None => format!("uid {me}"),
    };
    Health::Degraded(format!(
        "the command lines of the processes of every account other than {named} are not read, \
         and a program of theirs whose file was deleted is not named: macOS shows the arguments \
         of a process only to its own account and to root; run as root"
    ))
}
