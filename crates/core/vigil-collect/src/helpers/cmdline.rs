#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redacted {
    pub text: String,
    pub redacted: bool,
}

const HIDDEN: &str = "[redacted]";

const SECRET_FLAGS: &[&str] = &[
    "--password",
    "--pass",
    "--header",
    "--token",
    "--api-key",
    "--secret",
];

const SHORT_SECRET_FLAGS: &[&str] = &["-H"];

const USER_FLAG: &str = "--user";

const SHORT_USER_FLAG: &str = "-u";

const GLUED_PASSWORD_FLAG: &str = "-p";

const SECRET_ASSIGNMENTS: &[&str] = &[
    "password=",
    "passwd=",
    "pass=",
    "token=",
    "secret=",
    "api_key=",
    "apikey=",
    "access_key=",
    "authorization=",
    "auth=",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Next {
    Shown,
    Secret,
    User,
}

pub fn redact(arguments: &[String]) -> Redacted {
    let mut out: Vec<String> = Vec::with_capacity(arguments.len());
    let mut redacted = false;
    let mut next = Next::Shown;

    for argument in arguments {
        let hidden = match std::mem::replace(&mut next, Next::Shown) {
            Next::Secret => true,
            Next::User => holds_a_password(argument),
            Next::Shown => false,
        };
        if hidden {
            out.push(HIDDEN.to_string());
            redacted = true;
            continue;
        }

        if let Some(glued) = glued(argument) {
            out.push(glued);
            redacted = true;
            continue;
        }

        let lower = argument.to_ascii_lowercase();
        let unquoted = lower.trim_start_matches(['"', '\'']);
        if let Some(assignment) = SECRET_ASSIGNMENTS
            .iter()
            .find(|name| unquoted.starts_with(*name) && unquoted.len() > name.len())
        {
            out.push(format!("{assignment}{HIDDEN}"));
            redacted = true;
            continue;
        }

        if let Some((flag, value)) = argument.split_once('=')
            && (is_secret_flag(flag) || (is_user_flag(flag) && holds_a_password(value)))
        {
            out.push(format!("{flag}={HIDDEN}"));
            redacted = true;
            continue;
        }

        next = match () {
            _ if is_secret_flag(argument) => Next::Secret,
            _ if is_user_flag(argument) => Next::User,
            _ => Next::Shown,
        };
        out.push(argument.clone());
    }

    Redacted {
        text: out.join(" "),
        redacted,
    }
}

fn is_secret_flag(flag: &str) -> bool {
    SHORT_SECRET_FLAGS.contains(&flag)
        || SECRET_FLAGS
            .iter()
            .any(|known| known.eq_ignore_ascii_case(flag))
}

fn is_user_flag(flag: &str) -> bool {
    flag == SHORT_USER_FLAG || flag.eq_ignore_ascii_case(USER_FLAG)
}

fn holds_a_password(value: &str) -> bool {
    value.contains(':')
}

fn glued(argument: &str) -> Option<String> {
    if argument.len() <= 2 {
        return None;
    }
    if argument.starts_with(GLUED_PASSWORD_FLAG) {
        return Some(format!("{GLUED_PASSWORD_FLAG}{HIDDEN}"));
    }
    match argument.strip_prefix(SHORT_USER_FLAG) {
        Some(value) if holds_a_password(value) => Some(format!("{SHORT_USER_FLAG}{HIDDEN}")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(line: &[&str]) -> Vec<String> {
        line.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn a_port_number_after_minus_p_is_not_a_password() {
        let out = redact(&arguments(&["nc", "-l", "-p", "4444"]));

        assert_eq!(out.text, "nc -l -p 4444");
        assert!(!out.redacted);
    }

    #[test]
    fn hides_a_password_glued_to_its_flag() {
        let out = redact(&arguments(&["mysql", "-uroot", "-pS3cret", "shop"]));

        assert_eq!(out.text, "mysql -uroot -p[redacted] shop");
        assert!(out.redacted);
    }

    #[test]
    fn hides_the_argument_after_a_secret_flag() {
        let out = redact(&arguments(&[
            "curl",
            "-H",
            "Authorization: Bearer eyJhbGciOi",
            "https://api.example.com",
        ]));

        assert_eq!(out.text, "curl -H [redacted] https://api.example.com");
        assert!(!out.text.contains("eyJhbGciOi"));
    }

    #[test]
    fn hides_the_right_hand_side_of_an_assignment_in_either_form() {
        let out = redact(&arguments(&["app", "--password=hunter2", "TOKEN=abcdef"]));

        assert_eq!(out.text, "app --password=[redacted] token=[redacted]");
        assert!(out.redacted);
    }

    #[test]
    fn an_ordinary_command_line_comes_back_untouched_and_says_so() {
        let out = redact(&arguments(&["/usr/sbin/nginx", "-g", "daemon off;"]));

        assert_eq!(out.text, "/usr/sbin/nginx -g daemon off;");
        assert!(
            !out.redacted,
            "claiming a redaction that did not happen makes the flag useless"
        );
    }

    #[test]
    fn an_assignment_that_still_has_its_quotes_is_hidden_too() {
        let out = redact(&arguments(&["/bin/sh", "-c", "\"password=S3cret\""]));

        assert!(out.redacted);
        assert!(!out.text.contains("S3cret"), "{}", out.text);
    }
    #[test]
    fn a_user_without_a_password_is_a_name_and_stays_on_the_line() {
        for line in [
            &["sudo", "-u", "root", "id"][..],
            &["ps", "--user", "www-data"],
            &["mysql", "-uroot", "shop"],
            &["app", "--user=deploy"],
        ] {
            let out = redact(&arguments(line));
            assert_eq!(out.text, line.join(" "));
            assert!(!out.redacted, "{line:?}");
        }
    }

    #[test]
    fn a_user_with_a_password_after_a_colon_is_hidden_in_every_form() {
        for (line, shown) in [
            (
                &["curl", "-u", "admin:hunter2", "https://api.example.com"][..],
                "curl -u [redacted] https://api.example.com",
            ),
            (&["curl", "-uadmin:hunter2"], "curl -u[redacted]"),
            (&["wget", "--user=admin:hunter2"], "wget --user=[redacted]"),
            (
                &["wget", "--USER", "admin:hunter2"],
                "wget --USER [redacted]",
            ),
        ] {
            let out = redact(&arguments(line));
            assert_eq!(out.text, shown);
            assert!(!out.text.contains("hunter2"));
        }
    }

    #[test]
    fn a_short_flag_is_read_with_its_case_because_capitals_mean_other_things() {
        for line in [
            &["mysql", "-h", "db.internal", "-u", "app"][..],
            &["scp", "-P2222", "file", "host:"],
            &["tool", "-Uroot"],
        ] {
            let out = redact(&arguments(line));
            assert_eq!(out.text, line.join(" "), "{line:?}");
            assert!(!out.redacted, "{line:?}");
        }
    }

    #[test]
    fn a_long_flag_is_read_whatever_its_case() {
        let out = redact(&arguments(&["app", "--PASSWORD", "hunter2", "--Token=abc"]));

        assert_eq!(out.text, "app --PASSWORD [redacted] --Token=[redacted]");
    }
}
