pub fn unquoted(value: &str) -> String {
    let mut characters = value.chars();
    match (characters.next(), value.chars().last()) {
        (Some('"'), Some('"')) if value.chars().count() >= 2 => {
            let inside: String = value[1..value.len() - 1].to_string();
            inside.replace("\\\"", "\"").replace("\\\\", "\\")
        }
        (Some('\''), Some('\'')) if value.chars().count() >= 2 => {
            value[1..value.len() - 1].replace("''", "'")
        }
        _ => value.to_string(),
    }
}

pub fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_goes_into_the_file_quoted_and_comes_back_out_of_it_the_same() {
        for key in [
            "port.listen|tcp|0.0.0.0:4444",
            "user|sshkey|deploy|a\"b",
            "a\\b",
            "plain",
        ] {
            assert_eq!(unquoted(&quoted(key)), key, "{key}");
        }
    }

    #[test]
    fn what_somebody_wrote_by_hand_is_read_whichever_quotation_they_reached_for() {
        assert_eq!(unquoted("user|group|docker"), "user|group|docker");
        assert_eq!(unquoted("'user|group|docker'"), "user|group|docker");
        assert_eq!(unquoted("\"user|group|docker\""), "user|group|docker");
    }
}
