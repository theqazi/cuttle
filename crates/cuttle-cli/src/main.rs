//! `cuttle` binary entry point. Delegates to `cuttle_cli::run()` so the
//! interesting work happens in library code with full unit-test coverage.
//! This file stays small on purpose; logic added here is logic the test
//! suite cannot reach.

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut stdout_lock = stdout.lock();
    let mut stderr_lock = stderr.lock();
    let code = cuttle_cli::run(&argv, &mut stdout_lock, &mut stderr_lock);
    let _ = stdout_lock.flush();
    let _ = stderr_lock.flush();
    ExitCode::from(code as u8)
}
