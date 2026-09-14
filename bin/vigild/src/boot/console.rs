use crate::Config;
use crate::helpers::rfc3339;
use crate::socket::{self, Shared};
use crate::types::Schedule;

pub fn listen(config: &Config, schedule: &Schedule, shared: &Shared) -> Result<(), String> {
    socket::listen(&config.socket_path, shared.clone(), rfc3339::now)?;
    eprintln!("  console: {} (0600, this user only)", config.socket_path);
    eprintln!("  console: {}", killing(config));

    eprintln!(
        "  history: {}, kept {} days",
        config.state_dir, config.retention_days
    );
    for (name, every_seconds) in schedule.periods() {
        eprintln!("  collector {name}: read every {every_seconds}s");
    }

    Ok(())
}

fn killing(config: &Config) -> &'static str {
    match config.killing.from_the_console {
        true => "may ask this agent to close a listening socket (killing.from_the_console)",
        false => "reads, and asks for nothing (killing.from_the_console is off)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_the_console_is_allowed_to_do_to_this_host_is_said_at_start_up_either_way() {
        let quiet = Config::default();
        let armed = Config {
            killing: crate::config::Killing {
                from_the_console: true,
            },
            ..Config::default()
        };

        assert!(
            killing(&armed).contains("close a listening socket"),
            "an operator reading the start-up of a host where the console can stop a service \
             must be told so there, not in the file they did not open"
        );
        assert!(killing(&quiet).contains("off"));
        for said in [killing(&armed), killing(&quiet)] {
            assert!(
                said.contains("killing.from_the_console"),
                "and the key that changes it is named, so the answer is one grep away: {said}"
            );
        }
    }
}
