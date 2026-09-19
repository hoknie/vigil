use std::process::ExitCode;

fn main() -> ExitCode {
    vigil_launches_spool::start(std::env::args().skip(1))
}
