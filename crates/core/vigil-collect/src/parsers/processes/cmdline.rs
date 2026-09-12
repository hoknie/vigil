#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redacted {
    pub text: String,
    pub redacted: bool,
}

const HIDDEN: &str = "[redacted]";

const SECRET_FLAGS: &[&str] = &[
    "--password",
    "--pass",
    "-H",
    "--header",
    "--token",
    "--api-key",
    "--secret",
    "-u",
    "--user",
];

const GLUED_SECRET_FLAGS: &[&str] = &["-p", "-u"];

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

pub fn redact(arguments: &[String]) -> Redacted {
    let mut out: Vec<String> = Vec::with_capacity(arguments.len());
    let mut redacted = false;
    let mut hide_next = false;

    for argument in arguments {
        if hide_next {
            out.push(HIDDEN.to_string());
            redacted = true;
            hide_next = false;
            continue;
        }

        let lower = argument.to_ascii_lowercase();

        if let Some(flag) = GLUED_SECRET_FLAGS
            .iter()
            .find(|flag| argument.len() > 2 && lower.starts_with(&flag.to_ascii_lowercase()))
        {
            out.push(format!("{flag}{HIDDEN}"));
            redacted = true;
            continue;
        }

        let unquoted = lower.trim_start_matches(['"', '\'']);
        if let Some(assignment) = SECRET_ASSIGNMENTS
            .iter()
            .find(|name| unquoted.starts_with(*name) && unquoted.len() > name.len())
        {
            out.push(format!("{assignment}{HIDDEN}"));
            redacted = true;
            continue;
        }

        if let Some((flag, _value)) = argument.split_once('=')
            && SECRET_FLAGS
                .iter()
                .any(|known| known.eq_ignore_ascii_case(flag))
        {
            out.push(format!("{flag}={HIDDEN}"));
            redacted = true;
            continue;
        }

        if SECRET_FLAGS
            .iter()
            .any(|flag| flag.eq_ignore_ascii_case(argument))
        {
            hide_next = true;
        }
        out.push(argument.clone());
    }

    Redacted {
        text: out.join(" "),
        redacted,
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

        assert_eq!(out.text, "mysql -u[redacted] -p[redacted] shop");
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
}
