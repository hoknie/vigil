use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswdEntry {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    pub home: String,
    pub shell: String,
}

pub fn parse_passwd_entries(text: &str) -> Vec<PasswdEntry> {
    let mut entries = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 7 {
            continue;
        }
        let (Ok(uid), Ok(gid)) = (fields[2].parse::<u32>(), fields[3].parse::<u32>()) else {
            continue;
        };

        entries.push(PasswdEntry {
            name: fields[0].to_string(),
            uid,
            gid,
            home: fields[5].to_string(),
            shell: fields[6].to_string(),
        });
    }

    entries
}

pub fn parse_passwd(text: &str) -> BTreeMap<u32, String> {
    let mut users = BTreeMap::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split(':');
        let (Some(name), Some(_password), Some(uid)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let Ok(uid) = uid.parse::<u32>() else {
            continue;
        };
        users.entry(uid).or_insert_with(|| name.to_string());
    }

    users
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWD: &str = "\
root:x:0:0:root:/root:/bin/bash
# a comment, and a blank line follow

daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin
www-data:x:33:33:www-data:/var/www:/usr/sbin/nologin
toor:x:0:0:alias:/root:/bin/bash
broken-line-without-enough-fields
postgres:x:not-a-number:117::/var/lib/postgresql:/bin/bash
";

    #[test]
    fn maps_the_uids_it_understands_and_walks_past_the_rest() {
        let users = parse_passwd(PASSWD);

        assert_eq!(users.get(&0).map(String::as_str), Some("root"));
        assert_eq!(users.get(&33).map(String::as_str), Some("www-data"));
        assert_eq!(
            users.len(),
            3,
            "the two malformed lines must not add entries"
        );
    }

    #[test]
    fn a_second_account_on_the_same_uid_does_not_rename_the_first() {
        let users = parse_passwd(PASSWD);

        assert_eq!(
            users.get(&0).map(String::as_str),
            Some("root"),
            "uid 0 is root even when an alias account shares it"
        );
    }

    #[test]
    fn an_alias_account_on_uid_zero_is_its_own_entry_not_a_duplicate_of_root() {
        let entries = parse_passwd_entries(PASSWD);

        let uid_zero: Vec<&str> = entries
            .iter()
            .filter(|entry| entry.uid == 0)
            .map(|entry| entry.name.as_str())
            .collect();
        assert_eq!(uid_zero, ["root", "toor"]);
    }

    #[test]
    fn carries_the_home_and_the_shell_and_nothing_from_gecos() {
        let entries = parse_passwd_entries(PASSWD);
        let www = entries
            .iter()
            .find(|entry| entry.name == "www-data")
            .expect("present");

        assert_eq!(www.uid, 33);
        assert_eq!(www.gid, 33);
        assert_eq!(www.home, "/var/www");
        assert_eq!(www.shell, "/usr/sbin/nologin");
    }

    #[test]
    fn a_line_that_is_not_an_account_does_not_become_one() {
        let entries = parse_passwd_entries(PASSWD);

        assert_eq!(entries.len(), 4, "{entries:?}");
        assert!(entries.iter().all(|entry| entry.name != "postgres"));
    }
}
