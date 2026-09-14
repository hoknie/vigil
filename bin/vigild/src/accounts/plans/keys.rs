use vigil_users::parse_authorized_keys;

pub fn without(text: &str, fingerprints: &[String]) -> (String, usize) {
    let mut taken = 0;
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| match fingerprint_of(line) {
            Some(fingerprint) if fingerprints.contains(&fingerprint) => {
                taken += 1;
                false
            }
            _ => true,
        })
        .collect();
    (joined(&kept), taken)
}

pub fn with(text: &str, lines: &[String]) -> String {
    let mut all: Vec<&str> = text.lines().collect();
    all.extend(lines.iter().map(|line| line.trim()));
    joined(&all)
}

pub fn restyled(
    text: &str,
    fingerprint: &str,
    options: Option<&str>,
    comment: Option<&str>,
) -> Option<String> {
    let mut found = false;
    let lines: Vec<String> = text
        .lines()
        .map(|line| {
            if found || fingerprint_of(line).as_deref() != Some(fingerprint) {
                return line.to_string();
            }
            match rewritten(line, fingerprint, options, comment) {
                Some(new) => {
                    found = true;
                    new
                }
                None => line.to_string(),
            }
        })
        .collect();

    match found {
        true => Some(joined(
            &lines.iter().map(String::as_str).collect::<Vec<&str>>(),
        )),
        false => None,
    }
}

fn rewritten(
    line: &str,
    fingerprint: &str,
    options: Option<&str>,
    comment: Option<&str>,
) -> Option<String> {
    let tokens = split_outside_quotes(line);
    let at = (0..tokens.len().saturating_sub(1)).find(|at| {
        parse_authorized_keys(&format!("{} {}", tokens[*at], tokens[*at + 1]))
            .first()
            .is_some_and(|key| key.fingerprint == fingerprint)
    })?;

    let before = tokens[..at].join(" ");
    let after = tokens[at + 2..].join(" ");
    let options = options.map(str::trim).unwrap_or(before.as_str());
    let comment = comment.map(str::trim).unwrap_or(after.as_str());

    Some(
        [options, &tokens[at], &tokens[at + 1], comment]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<&str>>()
            .join(" "),
    )
}

fn fingerprint_of(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    parse_authorized_keys(trimmed)
        .into_iter()
        .next()
        .map(|key| key.fingerprint)
}

fn split_outside_quotes(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for character in line.trim().chars() {
        match character {
            '"' => {
                quoted = !quoted;
                current.push(character);
            }
            space if space.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            other => current.push(other),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn joined(lines: &[&str]) -> String {
    match lines.is_empty() {
        true => String::new(),
        false => format!("{}\n", lines.join("\n")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr person@laptop";

    const OTHER: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRq ci@build";

    fn fingerprint(line: &str) -> String {
        fingerprint_of(line).expect("a key")
    }

    #[test]
    fn a_key_taken_away_leaves_every_other_line_of_the_file_as_it_was() {
        let text = format!("# managed by hand\n{KEY}\n\n{OTHER}\n");

        let (after, taken) = without(&text, &[fingerprint(KEY)]);

        assert_eq!(taken, 1);
        assert_eq!(after, format!("# managed by hand\n\n{OTHER}\n"));
    }

    #[test]
    fn a_fingerprint_the_file_does_not_hold_takes_nothing_and_says_so_by_its_count() {
        let (after, taken) = without(&format!("{KEY}\n"), &[fingerprint(OTHER)]);

        assert_eq!(taken, 0);
        assert_eq!(after, format!("{KEY}\n"));
    }

    #[test]
    fn a_key_added_goes_at_the_end_on_a_line_of_its_own() {
        assert_eq!(with(KEY, &[OTHER.to_string()]), format!("{KEY}\n{OTHER}\n"));
        assert_eq!(with("", &[OTHER.to_string()]), format!("{OTHER}\n"));
    }

    #[test]
    fn options_and_comment_are_rewritten_on_the_one_line_and_the_key_itself_is_not_touched() {
        let text = format!("from=\"10.0.0.0/8\",command=\"echo hi there\" {KEY}\n{OTHER}\n");

        let after = restyled(&text, &fingerprint(KEY), Some("no-pty"), None).expect("found");

        assert_eq!(
            after,
            format!("no-pty {KEY}\n{OTHER}\n"),
            "the options were replaced, the quoted space inside the old ones did not split \
             them, and the comment the reading had was kept"
        );
        assert_eq!(
            fingerprint_of(after.lines().next().unwrap_or_default()),
            Some(fingerprint(KEY))
        );
    }

    #[test]
    fn an_empty_option_clears_them_and_an_empty_comment_clears_it() {
        let text = format!("no-pty {KEY}\n");

        let after = restyled(&text, &fingerprint(KEY), Some(""), Some("")).expect("found");

        assert_eq!(
            after,
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr\n"
        );
    }

    #[test]
    fn a_key_the_file_no_longer_holds_is_not_restyled_into_existence() {
        assert_eq!(
            restyled(
                &format!("{OTHER}\n"),
                &fingerprint(KEY),
                Some("no-pty"),
                None
            ),
            None
        );
    }
}
