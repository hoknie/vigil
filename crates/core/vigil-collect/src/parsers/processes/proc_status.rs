#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStatus {
    pub name: String,
    pub uid: u32,
    pub parent: u32,
}

pub fn parse_status(text: &str) -> Option<ProcessStatus> {
    let mut name = None;
    let mut uid = None;
    let mut parent = None;

    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        match key {
            "Name" => name = Some(value.trim().to_string()),
            "Uid" => uid = value.split_whitespace().next()?.parse().ok(),
            "PPid" => parent = value.trim().parse().ok(),
            _ => {}
        }
        if name.is_some() && uid.is_some() && parent.is_some() {
            break;
        }
    }

    Some(ProcessStatus {
        name: name.unwrap_or_default(),
        uid: uid?,
        parent: parent?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_name_the_real_uid_and_the_parent() {
        let status = parse_status(
            "Name:\tnginx\n\
             Umask:\t0022\n\
             State:\tS (sleeping)\n\
             Tgid:\t812\n\
             Pid:\t812\n\
             PPid:\t1\n\
             TracerPid:\t0\n\
             Uid:\t33\t33\t33\t33\n",
        )
        .expect("parses");

        assert_eq!(status.name, "nginx");
        assert_eq!(status.parent, 1);
        assert_eq!(status.uid, 33);
    }

    #[test]
    fn a_name_with_a_bracket_in_it_does_not_shift_anything() {
        let status = parse_status("Name:\tevil) 0 0 0\nUid:\t1000\t1000\t1000\t1000\nPPid:\t812\n")
            .expect("parses");

        assert_eq!(status.name, "evil) 0 0 0");
        assert_eq!(status.uid, 1000);
        assert_eq!(status.parent, 812);
    }

    #[test]
    fn a_record_that_ended_mid_read_is_not_half_a_process() {
        assert_eq!(parse_status("Name:\tsh\n"), None);
        assert_eq!(parse_status(""), None);
    }
}
