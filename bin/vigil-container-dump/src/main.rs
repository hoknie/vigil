use std::process::ExitCode;

fn main() -> ExitCode {
    vigil_container_dump::start(std::env::args().skip(1))
}
