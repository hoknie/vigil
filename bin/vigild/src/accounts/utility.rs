#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Utility {
    Usermod,
    Userdel,
    Groupadd,
    Groupmod,
    Groupdel,
    Gpasswd,
    Visudo,
    Loginctl,
}

impl Utility {
    #[cfg(test)]
    pub const ALL: &'static [Utility] = &[
        Utility::Usermod,
        Utility::Userdel,
        Utility::Groupadd,
        Utility::Groupmod,
        Utility::Groupdel,
        Utility::Gpasswd,
        Utility::Visudo,
        Utility::Loginctl,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Utility::Usermod => "usermod",
            Utility::Userdel => "userdel",
            Utility::Groupadd => "groupadd",
            Utility::Groupmod => "groupmod",
            Utility::Groupdel => "groupdel",
            Utility::Gpasswd => "gpasswd",
            Utility::Visudo => "visudo",
            Utility::Loginctl => "loginctl",
        }
    }

    pub fn places(self) -> &'static [&'static str] {
        match self {
            Utility::Usermod => &["/usr/sbin/usermod", "/sbin/usermod"],
            Utility::Userdel => &["/usr/sbin/userdel", "/sbin/userdel"],
            Utility::Groupadd => &["/usr/sbin/groupadd", "/sbin/groupadd"],
            Utility::Groupmod => &["/usr/sbin/groupmod", "/sbin/groupmod"],
            Utility::Groupdel => &["/usr/sbin/groupdel", "/sbin/groupdel"],
            Utility::Gpasswd => &["/usr/bin/gpasswd", "/usr/sbin/gpasswd", "/bin/gpasswd"],
            Utility::Visudo => &["/usr/sbin/visudo", "/usr/bin/visudo", "/sbin/visudo"],
            Utility::Loginctl => &["/usr/bin/loginctl", "/bin/loginctl"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_account_tool_is_looked_for_by_absolute_path_under_its_own_name_and_nowhere_else() {
        for utility in Utility::ALL {
            assert!(!utility.places().is_empty(), "{}", utility.name());
            for place in utility.places() {
                assert!(
                    place.starts_with('/') && place.ends_with(&format!("/{}", utility.name())),
                    "{place}: a tool found through PATH is whatever the environment of the \
                     daemon puts first, and this daemon runs as root"
                );
            }
        }
    }
}
