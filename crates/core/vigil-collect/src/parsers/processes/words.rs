pub fn split_command(command: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for character in command.chars() {
        if escaped {
            word.push(character);
            escaped = false;
            continue;
        }
        match (character, quote) {
            ('\\', None) => {
                word.push(character);
                escaped = true;
            }
            ('\'' | '"', None) => {
                word.push(character);
                quote = Some(character);
            }
            (c, Some(open)) if c == open => {
                word.push(character);
                quote = None;
            }
            (c, None) if c.is_whitespace() => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            _ => word.push(character),
        }
    }

    if !word.is_empty() {
        words.push(word);
    }

    words
}

#[cfg(test)]
mod tests {
    use super::super::cmdline::redact;
    use super::*;

    #[test]
    fn a_quoted_argument_stays_one_argument() {
        assert_eq!(
            split_command("/usr/bin/curl -H \"Authorization: Bearer abc\" http://x"),
            vec![
                "/usr/bin/curl",
                "-H",
                "\"Authorization: Bearer abc\"",
                "http://x"
            ]
        );
    }

    #[test]
    fn the_leak_this_module_was_written_for_does_not_happen_any_more() {
        let clean = redact(&split_command(
            "/usr/bin/curl -H \"Authorization: Bearer s3cret\" http://x",
        ));

        assert!(clean.redacted);
        assert!(!clean.text.contains("s3cret"), "{}", clean.text);
    }

    #[test]
    fn a_shell_command_keeps_its_quoting_so_the_snapshot_says_what_runs() {
        assert_eq!(
            split_command("/bin/sh -c 'a b'"),
            vec!["/bin/sh", "-c", "'a b'"]
        );
    }

    #[test]
    fn an_escaped_space_does_not_split_a_path() {
        assert_eq!(
            split_command("/opt/my\\ app/run --once"),
            vec!["/opt/my\\ app/run", "--once"]
        );
    }

    #[test]
    fn an_unterminated_quote_keeps_the_rest_of_the_line_rather_than_losing_it() {
        assert_eq!(
            split_command("/bin/echo \"unfinished"),
            vec!["/bin/echo", "\"unfinished"]
        );
    }
}
