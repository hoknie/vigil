#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Launchd,
    Unit,
    Timer,
    Cron,
    Module,
    ModulesUnreadable,
    Script,
    Preload,
    Unknown,
}

impl Kind {
    pub const NAMED: &'static [&'static str] = &[
        "launchd", "unit", "timer", "cron", "module", "modules", "script", "preload",
    ];

    pub fn of(key: &str) -> Kind {
        match key.split('|').next().unwrap_or_default() {
            "launchd" => Kind::Launchd,
            "unit" => Kind::Unit,
            "timer" => Kind::Timer,
            "cron" => Kind::Cron,
            "module" => Kind::Module,
            "modules" => Kind::ModulesUnreadable,
            "script" => Kind::Script,
            "preload" => Kind::Preload,
            _ => Kind::Unknown,
        }
    }

    pub fn mark(self) -> bool {
        self == Kind::ModulesUnreadable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_row_that_says_the_modules_were_not_readable_is_not_a_loaded_module() {
        assert!(Kind::ModulesUnreadable.mark());
        assert!(!Kind::Module.mark());
        assert_eq!(Kind::of("modules|unreadable"), Kind::ModulesUnreadable);
        assert_eq!(Kind::of("module|overlay"), Kind::Module);
    }

    #[test]
    fn every_word_a_known_key_starts_with_is_named_and_names_a_different_kind() {
        let mut kinds: Vec<Kind> = Vec::new();
        for name in Kind::NAMED {
            let kind = Kind::of(name);
            assert_ne!(
                kind,
                Kind::Unknown,
                "{name} is named as known, so the search for unknown keys skips it"
            );
            assert_eq!(Kind::of(&format!("{name}|x")), kind);
            assert!(!kinds.contains(&kind), "{name} names a kind twice");
            kinds.push(kind);
        }
        assert_eq!(
            kinds.len(),
            8,
            "a kind missing from the list would be searched for as unknown and never found"
        );
    }

    #[test]
    fn a_bare_word_is_its_kind_and_a_longer_word_starting_the_same_is_not() {
        assert_eq!(Kind::of("unit"), Kind::Unit);
        assert_eq!(Kind::of("units|x"), Kind::Unknown);
        assert_eq!(Kind::of("unit}x"), Kind::Unknown);
    }

    #[test]
    fn a_key_shaped_like_nothing_this_console_knows_is_still_a_kind_and_not_a_panic() {
        assert_eq!(Kind::of("initramfs|/boot/initrd"), Kind::Unknown);
        assert_eq!(Kind::of(""), Kind::Unknown);
    }
}
