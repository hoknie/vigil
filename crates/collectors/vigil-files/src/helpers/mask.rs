const SPECIAL: &[char] = &['*', '?', '['];

pub fn is_a_mask(component: &str) -> bool {
    component.contains(SPECIAL)
}

pub fn holds_a_mask(path: &str) -> bool {
    path.split('/').any(is_a_mask)
}

pub fn check_mask(path: &str) -> Result<(), String> {
    for component in path.split('/').filter(|component| is_a_mask(component)) {
        let characters: Vec<char> = component.chars().collect();
        let mut at = 0;
        while at < characters.len() {
            if characters[at] == '[' {
                at = class_end(&characters, at).ok_or_else(|| {
                    format!(
                        "{path:?}: the [ in {component:?} is never closed, so the mask would \
                         match nothing anybody meant"
                    )
                })?;
            }
            at += 1;
        }
    }
    Ok(())
}

pub fn matches(mask: &str, name: &str) -> bool {
    let mask: Vec<char> = mask.chars().collect();
    let name: Vec<char> = name.chars().collect();
    matched(&mask, &name)
}

fn matched(mask: &[char], name: &[char]) -> bool {
    let (mut m, mut n) = (0, 0);
    let mut star: Option<(usize, usize)> = None;

    while n < name.len() {
        match mask.get(m) {
            Some('*') => {
                star = Some((m, n));
                m += 1;
                continue;
            }
            Some('?') => {
                m += 1;
                n += 1;
                continue;
            }
            Some('[') => {
                if let Some((end, inside)) = class_at(mask, m, name[n]) {
                    if inside {
                        m = end + 1;
                        n += 1;
                        continue;
                    }
                } else if name[n] == '[' {
                    m += 1;
                    n += 1;
                    continue;
                }
            }
            Some(literal) if *literal == name[n] => {
                m += 1;
                n += 1;
                continue;
            }
            _ => {}
        }
        match star {
            Some((star_at, from)) => {
                m = star_at + 1;
                n = from + 1;
                star = Some((star_at, from + 1));
            }
            None => return false,
        }
    }

    mask[m..].iter().all(|character| *character == '*')
}

fn class_end(mask: &[char], open: usize) -> Option<usize> {
    let mut at = open + 1;
    if matches!(mask.get(at), Some('!') | Some('^')) {
        at += 1;
    }
    if mask.get(at) == Some(&']') {
        at += 1;
    }
    while at < mask.len() {
        if mask[at] == ']' {
            return Some(at);
        }
        at += 1;
    }
    None
}

fn class_at(mask: &[char], open: usize, character: char) -> Option<(usize, bool)> {
    let end = class_end(mask, open)?;
    let mut at = open + 1;
    let negated = matches!(mask.get(at), Some('!') | Some('^'));
    if negated {
        at += 1;
    }

    let mut found = false;
    while at < end {
        let low = mask[at];
        if at + 2 < end && mask[at + 1] == '-' {
            let high = mask[at + 2];
            found |= low <= character && character <= high;
            at += 3;
        } else {
            found |= low == character;
            at += 1;
        }
    }
    Some((end, found != negated))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_star_stands_for_any_run_of_characters_inside_one_name() {
        assert!(matches("*.conf", "sshd.conf"));
        assert!(matches("*.conf", ".conf"));
        assert!(matches("*", "anything"));
        assert!(matches("a*b*c", "aXXbYYc"));
        assert!(!matches("*.conf", "sshd.config"));
        assert!(!matches("*.conf", "sshd_config"));
    }

    #[test]
    fn a_name_beginning_with_a_dot_is_matched_by_a_star_because_a_hidden_file_is_still_a_file() {
        assert!(
            matches("*.conf", ".hidden.conf"),
            "a shell leaves hidden files out of a star; a file dropped into a watched place \
             with a dot in front of its name is the first thing a watcher must not miss"
        );
    }

    #[test]
    fn a_question_mark_stands_for_exactly_one_character() {
        assert!(matches("tty?", "tty1"));
        assert!(!matches("tty?", "tty"));
        assert!(!matches("tty?", "tty12"));
    }

    #[test]
    fn a_class_in_brackets_stands_for_one_character_of_it_or_one_outside_it_when_negated() {
        assert!(matches("sd[ab]1", "sda1"));
        assert!(!matches("sd[ab]1", "sdc1"));
        assert!(matches("log[0-9]", "log7"));
        assert!(!matches("log[0-9]", "logx"));
        assert!(matches("log[!0-9]", "logx"));
        assert!(matches("log[^0-9]", "logx"));
        assert!(!matches("log[!0-9]", "log3"));
        assert!(matches("a[]]b", "a]b"));
    }

    #[test]
    fn a_bracket_that_is_never_closed_is_refused_rather_than_matched_as_a_letter() {
        let refusal = check_mask("/etc/ssh/[abc.conf").expect_err("must not be accepted");

        assert!(refusal.contains("never closed"), "{refusal}");
        assert!(check_mask("/etc/ssh/*.conf").is_ok());
        assert!(check_mask("/etc/[a-z]*/x").is_ok());
    }

    #[test]
    fn a_path_is_a_mask_only_when_one_of_its_names_holds_a_star_a_question_mark_or_a_bracket() {
        assert!(holds_a_mask("/etc/ssh/*.conf"));
        assert!(holds_a_mask("/etc/*/sshd"));
        assert!(!holds_a_mask("/etc/ssh/sshd_config"));
        assert!(!holds_a_mask("/etc/pam.d"));
    }

    #[test]
    fn a_long_name_against_many_stars_is_decided_without_trying_every_split() {
        let name = "a".repeat(4096);

        assert!(!matches("*a*a*a*a*a*a*a*b", &name));
        assert!(matches("*a*a*a*a*a*a*a*", &name));
    }
}
