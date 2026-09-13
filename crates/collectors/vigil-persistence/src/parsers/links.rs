#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnitLinks {
    pub wanted_by: Vec<String>,
    pub required_by: Vec<String>,
    pub wants: Vec<String>,
    pub requires: Vec<String>,
    pub part_of: Vec<String>,
}

const NAMES_PER_SETTING: usize = 64;

impl UnitLinks {
    pub fn named(&self) -> [(&'static str, &[String]); 5] {
        [
            ("wanted_by", &self.wanted_by),
            ("required_by", &self.required_by),
            ("wants", &self.wants),
            ("requires", &self.requires),
            ("part_of", &self.part_of),
        ]
    }

    pub fn add(&mut self, setting: Setting, value: &str) {
        let names = match setting {
            Setting::WantedBy => &mut self.wanted_by,
            Setting::RequiredBy => &mut self.required_by,
            Setting::Wants => &mut self.wants,
            Setting::Requires => &mut self.requires,
            Setting::PartOf => &mut self.part_of,
        };

        if value.trim().is_empty() {
            names.clear();
            return;
        }

        for name in value.split_whitespace() {
            if names.len() >= NAMES_PER_SETTING {
                return;
            }
            if !names.iter().any(|held| held == name) {
                names.push(name.to_string());
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    WantedBy,
    RequiredBy,
    Wants,
    Requires,
    PartOf,
}

impl Setting {
    pub fn of(section: &str, key: &str) -> Option<Setting> {
        match (section, key) {
            ("Install", "WantedBy") => Some(Setting::WantedBy),
            ("Install", "RequiredBy") => Some(Setting::RequiredBy),
            ("Unit", "Wants") => Some(Setting::Wants),
            ("Unit", "Requires") => Some(Setting::Requires),
            ("Unit", "PartOf") => Some(Setting::PartOf),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_setting_naming_several_units_is_several_names_and_not_one() {
        let mut links = UnitLinks::default();
        links.add(Setting::WantedBy, "multi-user.target sockets.target");

        assert_eq!(
            links.wanted_by,
            vec![
                "multi-user.target".to_string(),
                "sockets.target".to_string()
            ]
        );
    }

    #[test]
    fn the_setting_written_twice_adds_rather_than_replaces_the_way_systemd_reads_it() {
        let mut links = UnitLinks::default();
        links.add(Setting::Wants, "a.service");
        links.add(Setting::Wants, "b.service a.service");

        assert_eq!(
            links.wants,
            vec!["a.service".to_string(), "b.service".to_string()],
            "and the same name twice is one edge, not two"
        );
    }

    #[test]
    fn an_empty_setting_wipes_the_list_the_way_systemd_reads_it() {
        let mut links = UnitLinks::default();
        links.add(Setting::Requires, "a.service");
        links.add(Setting::Requires, "");

        assert!(links.requires.is_empty());
    }

    #[test]
    fn a_file_naming_more_units_than_we_will_hold_stops_rather_than_grows_without_end() {
        let mut links = UnitLinks::default();
        for number in 0..NAMES_PER_SETTING * 4 {
            links.add(Setting::Wants, &format!("unit-{number}.service"));
        }

        assert_eq!(links.wants.len(), NAMES_PER_SETTING);
    }

    #[test]
    fn every_setting_that_is_read_is_one_a_reader_of_the_snapshot_can_name() {
        let mut links = UnitLinks::default();
        for setting in [
            Setting::WantedBy,
            Setting::RequiredBy,
            Setting::Wants,
            Setting::Requires,
            Setting::PartOf,
        ] {
            links.add(setting, "one.service");
        }

        assert_eq!(
            links.named().iter().filter(|(_, at)| at.len() == 1).count(),
            5
        );
    }

    #[test]
    fn ordering_is_not_a_dependency_and_is_not_read_as_one() {
        assert_eq!(Setting::of("Unit", "After"), None);
        assert_eq!(Setting::of("Unit", "Before"), None);
        assert_eq!(Setting::of("Unit", "Wants"), Some(Setting::Wants));
    }

    #[test]
    fn a_setting_of_the_same_name_in_the_wrong_section_is_not_the_one_we_want() {
        assert_eq!(Setting::of("Unit", "WantedBy"), None);
        assert_eq!(Setting::of("Service", "PartOf"), None);
        assert_eq!(Setting::of("Install", "WantedBy"), Some(Setting::WantedBy));
    }
}
