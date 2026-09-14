pub fn without(text: &str, who: &str) -> (String, usize) {
    replacing(text, who, &[])
}

pub fn replacing(text: &str, who: &str, rules: &[String]) -> (String, usize) {
    let mut kept: Vec<String> = Vec::new();
    let mut found = 0;

    for group in logical(text) {
        if first_token(&group) != Some(who) {
            kept.extend(group.iter().map(|line| line.to_string()));
            continue;
        }
        if found == 0 {
            kept.extend(rules.iter().map(|rule| format!("{who} {}", rule.trim())));
        }
        found += 1;
    }

    let text = match kept.is_empty() {
        true => String::new(),
        false => format!("{}\n", kept.join("\n")),
    };
    (text, found)
}

pub fn holds_rules(text: &str) -> bool {
    text.lines()
        .map(str::trim)
        .any(|line| !line.is_empty() && (!line.starts_with('#') || line.starts_with("#include")))
}

fn logical(text: &str) -> Vec<Vec<&str>> {
    let mut groups: Vec<Vec<&str>> = Vec::new();
    let mut continuing = false;
    for line in text.lines() {
        match (continuing, groups.last_mut()) {
            (true, Some(group)) => group.push(line),
            _ => groups.push(vec![line]),
        }
        continuing = line.trim_end().ends_with('\\');
    }
    groups
}

fn first_token<'a>(group: &[&'a str]) -> Option<&'a str> {
    let first = group.first()?.trim_start();
    if first.starts_with('#') {
        return None;
    }
    first.split_whitespace().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "\
# deploy may restart the app
deploy ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart app, \\
    /usr/bin/systemctl status app
%ops ALL=(ALL) ALL
deploy ALL=(root) /usr/bin/journalctl
";

    #[test]
    fn a_grant_taken_out_takes_every_line_of_it_including_the_ones_a_backslash_joined() {
        let (after, found) = without(FILE, "deploy");

        assert_eq!(found, 2);
        assert_eq!(after, "# deploy may restart the app\n%ops ALL=(ALL) ALL\n");
    }

    #[test]
    fn a_grant_rewritten_stands_where_its_first_line_stood_and_its_other_lines_go() {
        let (after, found) = replacing(FILE, "deploy", &["ALL=(ALL) /usr/bin/id".to_string()]);

        assert_eq!(found, 2);
        assert_eq!(
            after,
            "# deploy may restart the app\ndeploy ALL=(ALL) /usr/bin/id\n%ops ALL=(ALL) ALL\n"
        );
    }

    #[test]
    fn a_line_that_only_mentions_the_name_is_not_its_grant() {
        let (after, found) = without(FILE, "ops");

        assert_eq!(
            found, 0,
            "%ops is a group, and ops an account nobody named here"
        );
        assert_eq!(after, FILE);
        assert_eq!(without(FILE, "%ops").1, 1);
    }

    #[test]
    fn a_file_left_with_only_comments_holds_no_rules_and_an_include_is_a_rule() {
        assert!(!holds_rules("# nothing here\n\n"));
        assert!(holds_rules("Defaults env_reset\n"));
        assert!(holds_rules("#includedir /etc/sudoers.local\n"));
    }
}
