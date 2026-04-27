//! Cuttle top-level CLI library.
//!
//! Cargo binary `cuttle` (in `src/main.rs`) calls `cuttle_cli::run()` with
//! the process argv. Library shape exists so the argv parser + every
//! subcommand handler are unit-testable without spawning a subprocess.
//!
//! v0.0.11 surface:
//! - `cuttle --help` / `cuttle --version`.
//! - `cuttle telemetry [--json] [--falsifier-eval] [--audit-log <PATH>]`.
//!
//! Deferred to v0.0.12+:
//! - `cuttle session start` (interactive REPL; needs cuttle-anthropic
//!   streaming = its v0.0.9 milestone).
//! - `cuttle config show` / `cuttle config init`.
//! - `cuttle audit verify` (composes `cuttle_audit::verify_chain`).
//! - Full path defaults via `XDG_CONFIG_HOME` + per-day audit log files.
//!
//! Argv parser is hand-rolled. clap is widely audited but pulls a deep
//! dep tree; for the v0.0.11 surface (one subcommand, three flags), a
//! 60-line state-machine parser is faster to review and avoids the
//! supply-chain-tax-for-nothing trade-off CLAUDE.md §0e flags.

pub mod args;
pub mod paths;
pub mod telemetry_cmd;

use std::io::Write;

pub use args::{Cli, Command, ParseError, TelemetryArgs};

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("argument parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("telemetry subcommand failed: {0}")]
    Telemetry(#[from] telemetry_cmd::TelemetryCmdError),
}

/// CLI entry point. `argv` should INCLUDE the program name as `argv[0]`,
/// matching `std::env::args()`. Writes user-facing output to `stdout`
/// and diagnostics to `stderr`. Returns the process exit code: 0 on
/// success, 1 on usage error, 2 on subcommand failure.
pub fn run<W: Write, E: Write>(
    argv: &[String],
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let cli = match Cli::parse(argv) {
        Ok(c) => c,
        Err(ParseError::HelpRequested) => {
            let _ = writeln!(stdout, "{}", args::HELP_TEXT);
            return 0;
        }
        Err(ParseError::VersionRequested) => {
            let _ = writeln!(stdout, "cuttle {}", env!("CARGO_PKG_VERSION"));
            return 0;
        }
        Err(e) => {
            let _ = writeln!(stderr, "cuttle: {e}");
            let _ = writeln!(stderr, "Run `cuttle --help` for usage.");
            return 1;
        }
    };

    match cli.command {
        Command::Telemetry(args) => match telemetry_cmd::run(&args, stdout) {
            Ok(()) => 0,
            Err(e) => {
                let _ = writeln!(stderr, "cuttle telemetry: {e}");
                2
            }
        },
    }
}
