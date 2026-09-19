const PART_OF_A_NAME: [char; 4] = ['/', '.', '-', '_'];

const ENDS_A_DIRECTORY: [char; 5] = ['/', '\n', ' ', '"', '\''];

pub fn parent_of(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(parent, _)| parent)
        .filter(|parent| !parent.is_empty())
        .unwrap_or(path)
}

pub fn directory_renamed(text: &str, from: &str, to: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(from) {
        let after = &rest[at + from.len()..];
        let ends = after
            .chars()
            .next()
            .is_none_or(|next| ENDS_A_DIRECTORY.contains(&next));
        let begins = rest[..at]
            .chars()
            .last()
            .is_none_or(|before| !before.is_alphanumeric() && !PART_OF_A_NAME.contains(&before));
        out.push_str(&rest[..at]);
        out.push_str(match ends && begins {
            true => to,
            false => from,
        });
        rest = after;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_is_renamed_where_it_is_the_whole_of_a_path_or_the_head_of_one() {
        assert_eq!(
            directory_renamed("a: /etc/vigil\nb: /etc/vigil/x\n", "/etc/vigil", "/opt/v"),
            "a: /opt/v\nb: /opt/v/x\n"
        );
    }

    #[test]
    fn a_longer_name_or_a_deeper_path_that_holds_the_same_letters_is_left_alone() {
        let text = "/etc/vigilance /srv/etc/vigil /etc/vigil-old";

        assert_eq!(directory_renamed(text, "/etc/vigil", "/opt/v"), text);
    }

    #[test]
    fn the_parent_of_a_file_is_its_directory_and_a_bare_name_is_its_own() {
        assert_eq!(parent_of("/run/vigil/vigil.sock"), "/run/vigil");
        assert_eq!(parent_of("vigil.sock"), "vigil.sock");
        assert_eq!(parent_of("/vigil.sock"), "/vigil.sock");
    }
}
