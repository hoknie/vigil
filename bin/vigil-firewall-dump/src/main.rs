use std::process::ExitCode;

fn main() -> ExitCode {
    vigil_firewall_dump::start(std::env::args().skip(1))
}
