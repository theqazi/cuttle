//! Hand-rolled argv parser for `cuttle`.
//!
//! Grammar v0.0.11:
//!   cuttle [--help|-h] [--version|-V]
//!   cuttle telemetry [--json] [--falsifier-eval] [--audit-log <PATH>]
//!
//! Trade-off vs `clap`: this file is ~140 lines (with tests) and has
//! zero supply-chain attack surface beyond `std`. clap would buy us
//! prettier `--help` output + auto-generated completions; v0.0.11 doesn't
//! need either. Re-evaluate when the surface grows past ~5 subcommands
//! (per CLAUDE.md §0e simplicity-is-earned).

use std::path::PathBuf;
use thiserror::Error;

pub const HELP_TEXT: &str = "\
cuttle - security-first BYOK Anthropic Claude harness

USAGE:
    cuttle [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help       Show this help message and exit
    -V, --version    Show version and exit

SUBCOMMANDS:
    telemetry    Show local aggregate signal from the audit log

Run `cuttle <subcommand> --help` for subcommand-specific help.

TELEMETRY OPTIONS:
    --json                  Emit JSON instead of human-readable text
    --falsifier-eval        Additionally evaluate the v0.1 sealed-falsifier
                            predicates (DISABLE / SNAPSHOT-DRIFT / MEMORY-DRIFT)
    --audit-log <PATH>      Audit log file to read (default: ~/.cuttle/audit.jsonl)
";

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// Caller asked for help; not a real error, but propagated through
    /// `Result` so the caller short-circuits cleanly.
    #[error("help requested")]
    HelpRequested,
    #[error("version requested")]
    VersionRequested,
    #[error("missing subcommand")]
    MissingSubcommand,
    #[error("unknown subcommand: {0}")]
    UnknownSubcommand(String),
    #[error("unknown option: {0}")]
    UnknownOption(String),
    #[error("missing value for option {0}")]
    MissingValue(&'static str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cli {
    pub command: Command,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Telemetry(TelemetryArgs),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TelemetryArgs {
    pub json: bool,
    pub falsifier_eval: bool,
    pub audit_log: Option<PathBuf>,
}

impl Cli {
    /// Parse argv. `argv[0]` is the program name and ignored, matching
    /// the std::env::args() convention.
    pub fn parse(argv: &[String]) -> Result<Self, ParseError> {
        // Skip argv[0]. If nothing else, missing subcommand.
        let mut iter = argv.iter().skip(1);

        // Top-level options that short-circuit.
        let first = iter.next().ok_or(ParseError::MissingSubcommand)?;
        match first.as_str() {
            "-h" | "--help" => Err(ParseError::HelpRequested),
            "-V" | "--version" => Err(ParseError::VersionRequested),
            "telemetry" => {
                let args = parse_telemetry_args(&mut iter)?;
                Ok(Cli {
                    command: Command::Telemetry(args),
                })
            }
            other => Err(ParseError::UnknownSubcommand(other.to_string())),
        }
    }
}

fn parse_telemetry_args<'a, I>(iter: &mut I) -> Result<TelemetryArgs, ParseError>
where
    I: Iterator<Item = &'a String>,
{
    let mut args = TelemetryArgs::default();
    while let Some(tok) = iter.next() {
        match tok.as_str() {
            "--json" => args.json = true,
            "--falsifier-eval" => args.falsifier_eval = true,
            "--audit-log" => {
                let val = iter.next().ok_or(ParseError::MissingValue("--audit-log"))?;
                args.audit_log = Some(PathBuf::from(val));
            }
            "-h" | "--help" => return Err(ParseError::HelpRequested),
            other => return Err(ParseError::UnknownOption(other.to_string())),
        }
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(items: &[&str]) -> Vec<String> {
        std::iter::once("cuttle")
            .chain(items.iter().copied())
            .map(String::from)
            .collect()
    }

    #[test]
    fn no_args_is_missing_subcommand() {
        assert_eq!(Cli::parse(&argv(&[])), Err(ParseError::MissingSubcommand));
    }

    #[test]
    fn help_long_short_circuits() {
        assert_eq!(
            Cli::parse(&argv(&["--help"])),
            Err(ParseError::HelpRequested)
        );
    }

    #[test]
    fn help_short_short_circuits() {
        assert_eq!(Cli::parse(&argv(&["-h"])), Err(ParseError::HelpRequested));
    }

    #[test]
    fn version_long_short_circuits() {
        assert_eq!(
            Cli::parse(&argv(&["--version"])),
            Err(ParseError::VersionRequested)
        );
    }

    #[test]
    fn version_short_short_circuits() {
        assert_eq!(
            Cli::parse(&argv(&["-V"])),
            Err(ParseError::VersionRequested)
        );
    }

    #[test]
    fn unknown_subcommand_errors() {
        assert!(matches!(
            Cli::parse(&argv(&["bogus"])),
            Err(ParseError::UnknownSubcommand(s)) if s == "bogus"
        ));
    }

    #[test]
    fn telemetry_no_flags_uses_defaults() {
        let cli = Cli::parse(&argv(&["telemetry"])).unwrap();
        let Command::Telemetry(args) = cli.command;
        assert!(!args.json);
        assert!(!args.falsifier_eval);
        assert_eq!(args.audit_log, None);
    }

    #[test]
    fn telemetry_json_flag_parses() {
        let cli = Cli::parse(&argv(&["telemetry", "--json"])).unwrap();
        let Command::Telemetry(args) = cli.command;
        assert!(args.json);
    }

    #[test]
    fn telemetry_falsifier_eval_flag_parses() {
        let cli = Cli::parse(&argv(&["telemetry", "--falsifier-eval"])).unwrap();
        let Command::Telemetry(args) = cli.command;
        assert!(args.falsifier_eval);
    }

    #[test]
    fn telemetry_audit_log_path_parses() {
        let cli = Cli::parse(&argv(&["telemetry", "--audit-log", "/tmp/x.jsonl"])).unwrap();
        let Command::Telemetry(args) = cli.command;
        assert_eq!(args.audit_log, Some(PathBuf::from("/tmp/x.jsonl")));
    }

    #[test]
    fn telemetry_audit_log_missing_value_errors() {
        assert_eq!(
            Cli::parse(&argv(&["telemetry", "--audit-log"])),
            Err(ParseError::MissingValue("--audit-log"))
        );
    }

    #[test]
    fn telemetry_unknown_flag_errors() {
        assert!(matches!(
            Cli::parse(&argv(&["telemetry", "--bogus"])),
            Err(ParseError::UnknownOption(s)) if s == "--bogus"
        ));
    }

    #[test]
    fn telemetry_combines_flags() {
        let cli = Cli::parse(&argv(&["telemetry", "--json", "--falsifier-eval"])).unwrap();
        let Command::Telemetry(args) = cli.command;
        assert!(args.json);
        assert!(args.falsifier_eval);
    }

    #[test]
    fn telemetry_subcommand_help_short_circuits() {
        assert_eq!(
            Cli::parse(&argv(&["telemetry", "--help"])),
            Err(ParseError::HelpRequested)
        );
    }
}
