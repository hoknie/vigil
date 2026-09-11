use crate::Config;
use crate::helpers::rfc3339;
use crate::socket::{self, Shared};
use crate::types::Schedule;

pub fn listen(config: &Config, schedule: &Schedule, shared: &Shared) -> Result<(), String> {
    socket::listen(&config.socket_path, shared.clone(), rfc3339::now)?;
    eprintln!("  console: {} (0600, this user only)", config.socket_path);

    eprintln!(
        "  history: {}, kept {} days",
        config.state_dir, config.retention_days
    );
    for (name, every_seconds) in schedule.periods() {
        eprintln!("  collector {name}: read every {every_seconds}s");
    }

    Ok(())
}
