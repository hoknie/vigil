use serde_json::Value;

const MOST: usize = 255;

const THE_DAEMON: &str = "vigild.service";

pub fn unit(key: &str, item: &Value) -> Result<String, String> {
    let named = item["name"]
        .as_str()
        .or_else(|| key.split_once('|').map(|(_, name)| name))
        .unwrap_or_default();

    allowed(named)?;
    if ours(named) {
        return Err(format!(
            "{named} is a part of this agent: an agent asked to stop watching stops \
             watching, and nobody is told that it did. `vigild collector <name> disable` at a \
             shell of this host is where that is switched off"
        ));
    }
    Ok(named.to_string())
}

fn ours(named: &str) -> bool {
    named == THE_DAEMON
        || crate::modules::names()
            .into_iter()
            .any(|collector| crate::modules::unit_of(collector) == Some(named))
}

fn allowed(named: &str) -> Result<(), String> {
    if named.is_empty() {
        return Err(
            "the reading holds no name for this row, so there is nothing to name to \
                    systemctl"
                .to_string(),
        );
    }
    if named.len() > MOST {
        return Err(format!(
            "a unit name is at most {MOST} characters and this one is {}",
            named.len()
        ));
    }
    if named.starts_with('-') {
        return Err(format!(
            "{named} starts with a dash, which systemctl would read as an option of its own"
        ));
    }
    if !named.contains('.') {
        return Err(format!(
            "{named} carries no suffix, and a name without one is not a unit systemd has"
        ));
    }
    match named.chars().find(|letter| !holds(*letter)) {
        Some(letter) => Err(format!(
            "{named} holds {letter:?}, which is not a character of a unit name: this row was \
             not written by a reading of this host"
        )),
        None => Ok(()),
    }
}

fn holds(letter: char) -> bool {
    letter.is_ascii_alphanumeric() || ":-_.\\@".contains(letter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(name: &str) -> Value {
        json!({ "name": name, "path": format!("/lib/systemd/system/{name}") })
    }

    #[test]
    fn the_name_systemctl_is_given_comes_from_the_reading_and_not_from_the_key_it_was_asked_by() {
        assert_eq!(
            unit("unit|nginx.service", &item("nginx.service")),
            Ok("nginx.service".to_string())
        );
        assert_eq!(
            unit("timer|certbot.timer", &item("certbot.timer")),
            Ok("certbot.timer".to_string())
        );
        assert_eq!(
            unit("unit|getty@tty1.service", &item("getty@tty1.service")),
            Ok("getty@tty1.service".to_string()),
            "a template instance is a unit like any other, and refusing the @ would leave \
             every getty on this host unreachable from the console"
        );
    }

    #[test]
    fn a_name_that_could_be_read_as_something_other_than_a_unit_is_refused_by_name() {
        for named in [
            "-f",
            "nginx.service; rm -rf /",
            "../../etc/passwd",
            "nginx service",
            "nginx",
            "",
            "nginx.service\n--now",
        ] {
            let refused = unit("unit|x", &item(named))
                .expect_err("this daemon runs as root and hands this word to a program");
            assert!(
                !refused.is_empty(),
                "{named:?} was accepted: the argument list is built by this agent, and a name \
                 it did not read is a name somebody else chose"
            );
        }
    }

    #[test]
    fn neither_the_daemon_nor_a_unit_one_of_its_readings_needs_is_stopped_from_the_console() {
        let mut ours = vec![THE_DAEMON.to_string()];
        ours.extend(
            crate::modules::names()
                .into_iter()
                .filter_map(crate::modules::unit_of)
                .map(str::to_string),
        );

        assert!(
            ours.len() > 1,
            "this build installs a unit for at least one collector, and a list that came \
             back holding only the daemon means the collectors are no longer asked"
        );
        for named in ours {
            let refused = unit("unit|x", &item(&named)).expect_err("not this one");
            assert!(
                refused.contains("part of this agent"),
                "an agent that stopped its own reading at somebody's keystroke reports \
                 nothing afterwards, including that it was asked: {refused}"
            );
        }
    }

    #[test]
    fn a_row_with_no_name_in_it_falls_back_on_the_key_rather_than_on_an_empty_word() {
        assert_eq!(
            unit("unit|sshd.service", &json!({})),
            Ok("sshd.service".to_string())
        );
        assert!(unit("unit|", &json!({})).is_err());
    }
}
