use std::process::ExitCode;

fn main() -> ExitCode {
    vigil_audit_plugin::start(std::env::args().skip(1))
}
