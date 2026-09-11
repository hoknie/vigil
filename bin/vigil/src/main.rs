use std::process::ExitCode;

fn main() -> ExitCode {
    vigil::start(std::env::args())
}
