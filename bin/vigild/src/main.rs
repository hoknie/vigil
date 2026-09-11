use std::process::ExitCode;

fn main() -> ExitCode {
    vigild::start(std::env::args())
}
