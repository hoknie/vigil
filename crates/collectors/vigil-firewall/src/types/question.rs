pub const PFCTL: &str = "/sbin/pfctl";

pub const SOCKETFILTERFW: &str = "/usr/libexec/ApplicationFirewall/socketfilterfw";

pub const PF_INFO: &str = "pf info";

pub const PF_RULES: &str = "pf rules";

pub const PF_NAT: &str = "pf nat";

pub const PF_ANCHORS: &str = "pf anchors";

pub const ANCHOR_RULES: &str = "anchor rules ";

pub const ANCHOR_NAT: &str = "anchor nat ";

pub const APPLICATION_FIREWALL: &str = "application firewall";

pub const MOST_ANCHORS: usize = 64;

const LONGEST_ANCHOR: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub key: String,
    pub program: &'static str,
    pub arguments: Vec<String>,
}

impl Question {
    fn of(key: &str, program: &'static str, arguments: &[&str]) -> Question {
        Question {
            key: key.to_string(),
            program,
            arguments: arguments.iter().map(|word| (*word).to_string()).collect(),
        }
    }

    pub fn first() -> Vec<Question> {
        vec![
            Question::of(PF_INFO, PFCTL, &["-s", "info"]),
            Question::of(PF_RULES, PFCTL, &["-s", "rules"]),
            Question::of(PF_NAT, PFCTL, &["-s", "nat"]),
            Question::of(PF_ANCHORS, PFCTL, &["-v", "-s", "Anchors"]),
            Question::of(
                APPLICATION_FIREWALL,
                SOCKETFILTERFW,
                &[
                    "--getglobalstate",
                    "--getblockall",
                    "--getstealthmode",
                    "--getallowsigned",
                    "--listapps",
                ],
            ),
        ]
    }

    pub fn of_anchor(anchor: &str) -> Option<Vec<Question>> {
        if !is_an_anchor(anchor) {
            return None;
        }
        Some(vec![
            Question::of(
                &format!("{ANCHOR_RULES}{anchor}"),
                PFCTL,
                &["-a", anchor, "-s", "rules"],
            ),
            Question::of(
                &format!("{ANCHOR_NAT}{anchor}"),
                PFCTL,
                &["-a", anchor, "-s", "nat"],
            ),
        ])
    }
}

pub fn is_an_anchor(named: &str) -> bool {
    !named.is_empty()
        && named.len() <= LONGEST_ANCHOR
        && !named.starts_with(['-', '/'])
        && !named.contains("..")
        && named
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-/".contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHANGES_SOMETHING: &[&str] = &[
        "-e",
        "-E",
        "-d",
        "-X",
        "-f",
        "-F",
        "-k",
        "-K",
        "-t",
        "-T",
        "-D",
        "-i",
        "-o",
        "-x",
        "--setglobalstate",
        "--setblockall",
        "--setstealthmode",
        "--setallowsigned",
        "--add",
        "--remove",
        "--block",
        "--unblock",
        "--setloggingmode",
    ];

    #[test]
    fn every_question_the_dump_asks_reads_and_none_of_them_changes_anything() {
        let mut asked = Question::first();
        asked.extend(Question::of_anchor("com.apple/250.ApplicationFirewall").expect("a name"));

        for question in asked {
            assert!(question.program.starts_with('/'), "{question:?}");
            for word in &question.arguments {
                assert!(
                    !CHANGES_SOMETHING.contains(&word.as_str()),
                    "{question:?}: this job runs as root on a timer, and a flag that changes \
                     the firewall is a firewall changed every minute"
                );
            }
        }
    }

    #[test]
    fn an_anchor_is_asked_about_only_when_its_name_is_one_pf_could_have_given_it() {
        assert!(Question::of_anchor("com.apple/200.AirDrop/Bonjour").is_some());
        for hostile in [
            "",
            "-F",
            "-Fall",
            "/etc",
            "a/../b",
            "com.apple rules",
            "x;rm",
            "a\nb",
        ] {
            assert!(
                Question::of_anchor(hostile).is_none(),
                "{hostile:?}: a name read out of one answer becomes an argument of the next \
                 command, run as root"
            );
        }
    }

    #[test]
    fn the_programs_asked_are_the_two_every_mac_carries_at_one_path() {
        assert_eq!(PFCTL, "/sbin/pfctl");
        assert_eq!(
            SOCKETFILTERFW,
            "/usr/libexec/ApplicationFirewall/socketfilterfw"
        );
    }
}
