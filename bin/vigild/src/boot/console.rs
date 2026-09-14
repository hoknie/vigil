use crate::Config;
use crate::helpers::rfc3339;
use crate::socket::{self, Shared};
use crate::types::Schedule;

pub fn listen(config: &Config, schedule: &Schedule, shared: &Shared) -> Result<(), String> {
    socket::listen(&config.socket_path, shared.clone(), rfc3339::now)?;
    eprintln!("  console: {} (0600, this user only)", config.socket_path);
    eprintln!("  console: {}", killing(config));
    eprintln!("  console: {}", accounts(config));

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
        true => {
            "may ask this agent to stop a program or close a listening socket \
             (killing.from_the_console)"
        }
        false => "reads, and asks for nothing (killing.from_the_console is off)",
    }
}

fn accounts(config: &Config) -> &'static str {
    match config.accounts.from_the_console {
        true => {
            "may ask this agent to change accounts, groups, sudo grants and keys, and to end a \
             session (accounts.from_the_console)"
        }
        false => "changes no account on this host (accounts.from_the_console is off)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whether_the_console_may_change_accounts_is_said_at_start_up_either_way() {
        let quiet = Config::default();
        let armed = Config {
            accounts: crate::config::Accounts {
                from_the_console: true,
            },
            ..Config::default()
        };

        assert!(
            accounts(&armed).contains("sudo") && accounts(&armed).contains("accounts"),
            "an operator reading the start-up of a host where the console can hand out sudo \
             must be told so there: {}",
            accounts(&armed)
        );
        assert!(accounts(&quiet).contains("off"));
        for said in [accounts(&armed), accounts(&quiet)] {
            assert!(said.contains("accounts.from_the_console"), "{said}");
        }
    }

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
            killing(&armed).contains("close a listening socket")
                && killing(&armed).contains("stop a program"),
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
