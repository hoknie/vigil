#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
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
    pub fn of(key: &str) -> Kind {
        match key.split('|').next().unwrap_or_default() {
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
    fn a_key_shaped_like_nothing_this_console_knows_is_still_a_kind_and_not_a_panic() {
        assert_eq!(Kind::of("initramfs|/boot/initrd"), Kind::Unknown);
        assert_eq!(Kind::of(""), Kind::Unknown);
    }
}
