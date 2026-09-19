use vigil_collect::{CollectError, Collector, Health, NotOnThisSystem};
use vigil_model::{Rfc3339, Snapshot};

pub struct FirewallCollector {
    absent: NotOnThisSystem,
}

impl FirewallCollector {
    pub fn new(_now: impl Fn() -> Rfc3339 + Send + Sync + 'static, _counting: bool) -> Self {
        FirewallCollector {
            absent: NotOnThisSystem::new(
                "firewall",
                format!(
                    "the rules a host filters by are read from nftables on Linux and from pf \
                     and the Application Firewall on macOS, and this is {}",
                    std::env::consts::OS
                ),
            ),
        }
    }
}

impl Collector for FirewallCollector {
    fn name(&self) -> &'static str {
        self.absent.name()
    }

    fn available(&self) -> Health {
        self.absent.available()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        self.absent.collect()
    }
}
