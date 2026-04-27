//! Hand-rolled argv parser for `cuttle`.
//!
//! Grammar v0.0.15:
//!   cuttle [--help|-h] [--version|-V]
//!   cuttle telemetry [--json] [--falsifier-eval] [--audit-log <PATH>]
//!   cuttle ask [--model <MODEL>] [--max-tokens <N>]
//!              [--api-key-env <VAR>] [--stdin] [<PROMPT>]
//!   cuttle audit verify --audit-log <PATH> --chain-key-file <PATH>
//!   cuttle session start [--model <MODEL>] [--max-tokens <N>]
//!                         [--api-key-env <VAR>] [--system <SYSTEM>]
//!   cuttle sandbox profile [--project-root <PATH>]
//!   cuttle sandbox run [--project-root <PATH>] <PROGRAM> [ARGS...]
//!
//! Trade-off vs `clap`: this file is now ~280 lines (with tests). Still
//! zero supply-chain attack surface beyond `std`. clap will be worth
//! adopting when the subcommand surface crosses ~5 commands or when we
//! need POSIX `--` separator handling for argument forwarding.

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
    telemetry        Show local aggregate signal from the audit log
    ask              Send a single prompt to Claude (streaming response)
    audit verify     Verify an audit log's HMAC chain integrity
    session start    Start an interactive multi-turn REPL with audit chain
    sandbox profile  Print the SBPL profile cuttle would apply
    sandbox run      Run a command under the macOS sandbox (testing surface)

Run `cuttle <subcommand> --help` for subcommand-specific help.

TELEMETRY OPTIONS:
    --json                  Emit JSON instead of human-readable text
    --falsifier-eval        Additionally evaluate the v0.1 sealed-falsifier
                            predicates (DISABLE / SNAPSHOT-DRIFT / MEMORY-DRIFT)
    --audit-log <PATH>      Audit log file to read (default: ~/.cuttle/audit.jsonl)

ASK OPTIONS:
    --model <MODEL>         Model id (default: claude-sonnet-4-6)
    --max-tokens <N>        Maximum output tokens (default: 4096)
    --api-key-env <VAR>     Environment variable holding the API key
                            (default: ANTHROPIC_API_KEY)
    --stdin                 Read prompt from stdin instead of positional argument
    <PROMPT>                Prompt text (positional; required unless --stdin)

AUDIT VERIFY OPTIONS:
    --audit-log <PATH>          Audit log file to verify (required)
    --chain-key-file <PATH>     File containing the 32-byte session chain key
                                (raw 32 bytes OR 64 hex chars; required)

SESSION START OPTIONS:
    --model <MODEL>             Model id (default: claude-sonnet-4-6)
    --max-tokens <N>            Maximum output tokens per turn (default: 4096)
    --api-key-env <VAR>         Environment variable holding the API key
                                (default: ANTHROPIC_API_KEY)
    --system <SYSTEM>           System prompt applied to every turn (optional)

SANDBOX PROFILE OPTIONS:
    --project-root <PATH>       Absolute path used for the file-read/write subpaths
                                in the rendered SBPL (default: current directory)

SANDBOX RUN OPTIONS:
    --project-root <PATH>       Absolute path scoped as the sandbox's read+write
                                root (default: current directory)
    <PROGRAM> [ARGS...]         Program + args to execute under the sandbox.
                                The program must be in the default-allowed
                                binary set; otherwise the sandbox denies exec.
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
    #[error("invalid integer for option {opt}: {value}")]
    InvalidInt { opt: &'static str, value: String },
    #[error("missing required prompt; pass it as a positional argument or use --stdin")]
    MissingPrompt,
    #[error("--stdin and a positional <PROMPT> are mutually exclusive")]
    PromptAndStdin,
    #[error("missing required option {0}")]
    MissingRequired(&'static str),
    #[error("missing audit subcommand; expected `verify`")]
    MissingAuditSubcommand,
    #[error("unknown audit subcommand: {0}")]
    UnknownAuditSubcommand(String),
    #[error("missing session subcommand; expected `start`")]
    MissingSessionSubcommand,
    #[error("unknown session subcommand: {0}")]
    UnknownSessionSubcommand(String),
    #[error("missing sandbox subcommand; expected `profile` or `run`")]
    MissingSandboxSubcommand,
    #[error("unknown sandbox subcommand: {0}")]
    UnknownSandboxSubcommand(String),
    #[error("missing required <PROGRAM> for `cuttle sandbox run`")]
    MissingProgram,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cli {
    pub command: Command,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Telemetry(TelemetryArgs),
    Ask(AskArgs),
    AuditVerify(AuditVerifyArgs),
    SessionStart(SessionStartArgs),
    SandboxProfile(SandboxProfileArgs),
    SandboxRun(SandboxRunArgs),
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct SandboxProfileArgs {
    /// None means "use the current working directory at runtime".
    pub project_root: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SandboxRunArgs {
    pub project_root: Option<PathBuf>,
    pub program: PathBuf,
    pub args: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AuditVerifyArgs {
    pub audit_log: PathBuf,
    pub chain_key_file: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SessionStartArgs {
    pub model: String,
    pub max_tokens: u32,
    pub api_key_env: String,
    pub system: Option<String>,
}

impl Default for SessionStartArgs {
    fn default() -> Self {
        SessionStartArgs {
            model: "claude-sonnet-4-6".to_string(),
            max_tokens: 4096,
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
            system: None,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TelemetryArgs {
    pub json: bool,
    pub falsifier_eval: bool,
    pub audit_log: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AskArgs {
    pub model: String,
    pub max_tokens: u32,
    pub api_key_env: String,
    /// Either Some(prompt) (positional) OR Stdin (reader).
    pub source: PromptSource,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PromptSource {
    Inline(String),
    Stdin,
}

impl Default for AskArgs {
    fn default() -> Self {
        AskArgs {
            model: "claude-sonnet-4-6".to_string(),
            max_tokens: 4096,
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
            // Placeholder — real value populated during parse. Default
            // exists so test scaffolding can `..AskArgs::default()` cleanly.
            source: PromptSource::Stdin,
        }
    }
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
            "ask" => {
                let args = parse_ask_args(&mut iter)?;
                Ok(Cli {
                    command: Command::Ask(args),
                })
            }
            "audit" => {
                let sub = iter.next().ok_or(ParseError::MissingAuditSubcommand)?;
                match sub.as_str() {
                    "verify" => {
                        let args = parse_audit_verify_args(&mut iter)?;
                        Ok(Cli {
                            command: Command::AuditVerify(args),
                        })
                    }
                    "-h" | "--help" => Err(ParseError::HelpRequested),
                    other => Err(ParseError::UnknownAuditSubcommand(other.to_string())),
                }
            }
            "session" => {
                let sub = iter.next().ok_or(ParseError::MissingSessionSubcommand)?;
                match sub.as_str() {
                    "start" => {
                        let args = parse_session_start_args(&mut iter)?;
                        Ok(Cli {
                            command: Command::SessionStart(args),
                        })
                    }
                    "-h" | "--help" => Err(ParseError::HelpRequested),
                    other => Err(ParseError::UnknownSessionSubcommand(other.to_string())),
                }
            }
            "sandbox" => {
                let sub = iter.next().ok_or(ParseError::MissingSandboxSubcommand)?;
                match sub.as_str() {
                    "profile" => {
                        let args = parse_sandbox_profile_args(&mut iter)?;
                        Ok(Cli {
                            command: Command::SandboxProfile(args),
                        })
                    }
                    "run" => {
                        let args = parse_sandbox_run_args(&mut iter)?;
                        Ok(Cli {
                            command: Command::SandboxRun(args),
                        })
                    }
                    "-h" | "--help" => Err(ParseError::HelpRequested),
                    other => Err(ParseError::UnknownSandboxSubcommand(other.to_string())),
                }
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

fn parse_ask_args<'a, I>(iter: &mut I) -> Result<AskArgs, ParseError>
where
    I: Iterator<Item = &'a String>,
{
    let mut args = AskArgs::default();
    let mut stdin_flag = false;
    let mut positional_prompt: Option<String> = None;

    while let Some(tok) = iter.next() {
        match tok.as_str() {
            "--model" => {
                let val = iter.next().ok_or(ParseError::MissingValue("--model"))?;
                args.model = val.clone();
            }
            "--max-tokens" => {
                let val = iter
                    .next()
                    .ok_or(ParseError::MissingValue("--max-tokens"))?;
                args.max_tokens = val.parse::<u32>().map_err(|_| ParseError::InvalidInt {
                    opt: "--max-tokens",
                    value: val.clone(),
                })?;
            }
            "--api-key-env" => {
                let val = iter
                    .next()
                    .ok_or(ParseError::MissingValue("--api-key-env"))?;
                args.api_key_env = val.clone();
            }
            "--stdin" => stdin_flag = true,
            "-h" | "--help" => return Err(ParseError::HelpRequested),
            other if other.starts_with("--") => {
                return Err(ParseError::UnknownOption(other.to_string()));
            }
            other => {
                // Positional prompt. Concatenate multiple positional tokens
                // with spaces so `cuttle ask hello world` works without
                // requiring shell quoting around the whole prompt.
                match &mut positional_prompt {
                    Some(existing) => {
                        existing.push(' ');
                        existing.push_str(other);
                    }
                    None => positional_prompt = Some(other.to_string()),
                }
            }
        }
    }

    args.source = match (stdin_flag, positional_prompt) {
        (true, Some(_)) => return Err(ParseError::PromptAndStdin),
        (true, None) => PromptSource::Stdin,
        (false, Some(p)) => PromptSource::Inline(p),
        (false, None) => return Err(ParseError::MissingPrompt),
    };
    Ok(args)
}

fn parse_audit_verify_args<'a, I>(iter: &mut I) -> Result<AuditVerifyArgs, ParseError>
where
    I: Iterator<Item = &'a String>,
{
    let mut audit_log: Option<PathBuf> = None;
    let mut chain_key_file: Option<PathBuf> = None;

    while let Some(tok) = iter.next() {
        match tok.as_str() {
            "--audit-log" => {
                let val = iter.next().ok_or(ParseError::MissingValue("--audit-log"))?;
                audit_log = Some(PathBuf::from(val));
            }
            "--chain-key-file" => {
                let val = iter
                    .next()
                    .ok_or(ParseError::MissingValue("--chain-key-file"))?;
                chain_key_file = Some(PathBuf::from(val));
            }
            "-h" | "--help" => return Err(ParseError::HelpRequested),
            other => return Err(ParseError::UnknownOption(other.to_string())),
        }
    }
    Ok(AuditVerifyArgs {
        audit_log: audit_log.ok_or(ParseError::MissingRequired("--audit-log"))?,
        chain_key_file: chain_key_file.ok_or(ParseError::MissingRequired("--chain-key-file"))?,
    })
}

fn parse_sandbox_profile_args<'a, I>(iter: &mut I) -> Result<SandboxProfileArgs, ParseError>
where
    I: Iterator<Item = &'a String>,
{
    let mut args = SandboxProfileArgs::default();
    while let Some(tok) = iter.next() {
        match tok.as_str() {
            "--project-root" => {
                let val = iter
                    .next()
                    .ok_or(ParseError::MissingValue("--project-root"))?;
                args.project_root = Some(PathBuf::from(val));
            }
            "-h" | "--help" => return Err(ParseError::HelpRequested),
            other => return Err(ParseError::UnknownOption(other.to_string())),
        }
    }
    Ok(args)
}

fn parse_sandbox_run_args<'a, I>(iter: &mut I) -> Result<SandboxRunArgs, ParseError>
where
    I: Iterator<Item = &'a String>,
{
    let mut project_root: Option<PathBuf> = None;
    let mut program: Option<PathBuf> = None;
    let mut child_args: Vec<String> = Vec::new();

    while let Some(tok) = iter.next() {
        // Once we have the program, everything else is verbatim child args
        // (including things that look like flags). This lets the operator
        // pass `--foo` to the sandboxed program without the cuttle parser
        // grabbing it.
        if program.is_some() {
            child_args.push(tok.clone());
            continue;
        }
        match tok.as_str() {
            "--project-root" => {
                let val = iter
                    .next()
                    .ok_or(ParseError::MissingValue("--project-root"))?;
                project_root = Some(PathBuf::from(val));
            }
            "-h" | "--help" => return Err(ParseError::HelpRequested),
            "--" => {
                // Explicit separator: the next positional is the program.
                // Skip; the next loop iteration will pick up the program
                // via the `program.is_none()` branch below.
                continue;
            }
            other if other.starts_with("--") => {
                return Err(ParseError::UnknownOption(other.to_string()));
            }
            other => program = Some(PathBuf::from(other)),
        }
    }

    let program = program.ok_or(ParseError::MissingProgram)?;
    Ok(SandboxRunArgs {
        project_root,
        program,
        args: child_args,
    })
}

fn parse_session_start_args<'a, I>(iter: &mut I) -> Result<SessionStartArgs, ParseError>
where
    I: Iterator<Item = &'a String>,
{
    let mut args = SessionStartArgs::default();
    while let Some(tok) = iter.next() {
        match tok.as_str() {
            "--model" => {
                let val = iter.next().ok_or(ParseError::MissingValue("--model"))?;
                args.model = val.clone();
            }
            "--max-tokens" => {
                let val = iter
                    .next()
                    .ok_or(ParseError::MissingValue("--max-tokens"))?;
                args.max_tokens = val.parse::<u32>().map_err(|_| ParseError::InvalidInt {
                    opt: "--max-tokens",
                    value: val.clone(),
                })?;
            }
            "--api-key-env" => {
                let val = iter
                    .next()
                    .ok_or(ParseError::MissingValue("--api-key-env"))?;
                args.api_key_env = val.clone();
            }
            "--system" => {
                let val = iter.next().ok_or(ParseError::MissingValue("--system"))?;
                args.system = Some(val.clone());
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

    fn telemetry_of(cli: Cli) -> TelemetryArgs {
        match cli.command {
            Command::Telemetry(a) => a,
            other => panic!("expected Command::Telemetry, got {other:?}"),
        }
    }

    fn ask_of(cli: Cli) -> AskArgs {
        match cli.command {
            Command::Ask(a) => a,
            other => panic!("expected Command::Ask, got {other:?}"),
        }
    }

    fn audit_verify_of(cli: Cli) -> AuditVerifyArgs {
        match cli.command {
            Command::AuditVerify(a) => a,
            other => panic!("expected Command::AuditVerify, got {other:?}"),
        }
    }

    fn session_start_of(cli: Cli) -> SessionStartArgs {
        match cli.command {
            Command::SessionStart(a) => a,
            other => panic!("expected Command::SessionStart, got {other:?}"),
        }
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
        let args = telemetry_of(cli);
        assert!(!args.json);
        assert!(!args.falsifier_eval);
        assert_eq!(args.audit_log, None);
    }

    #[test]
    fn telemetry_json_flag_parses() {
        let cli = Cli::parse(&argv(&["telemetry", "--json"])).unwrap();
        let args = telemetry_of(cli);
        assert!(args.json);
    }

    #[test]
    fn telemetry_falsifier_eval_flag_parses() {
        let cli = Cli::parse(&argv(&["telemetry", "--falsifier-eval"])).unwrap();
        let args = telemetry_of(cli);
        assert!(args.falsifier_eval);
    }

    #[test]
    fn telemetry_audit_log_path_parses() {
        let cli = Cli::parse(&argv(&["telemetry", "--audit-log", "/tmp/x.jsonl"])).unwrap();
        let args = telemetry_of(cli);
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
        let args = telemetry_of(cli);
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

    #[test]
    fn ask_with_inline_prompt_uses_defaults() {
        let cli = Cli::parse(&argv(&["ask", "hello"])).unwrap();
        let args = ask_of(cli);
        assert_eq!(args.model, "claude-sonnet-4-6");
        assert_eq!(args.max_tokens, 4096);
        assert_eq!(args.api_key_env, "ANTHROPIC_API_KEY");
        assert_eq!(args.source, PromptSource::Inline("hello".into()));
    }

    #[test]
    fn ask_concatenates_multiple_positional_tokens() {
        let cli = Cli::parse(&argv(&["ask", "hello", "world"])).unwrap();
        let args = ask_of(cli);
        assert_eq!(args.source, PromptSource::Inline("hello world".into()));
    }

    #[test]
    fn ask_stdin_flag_selects_stdin_source() {
        let cli = Cli::parse(&argv(&["ask", "--stdin"])).unwrap();
        let args = ask_of(cli);
        assert_eq!(args.source, PromptSource::Stdin);
    }

    #[test]
    fn ask_model_override_parses() {
        let cli = Cli::parse(&argv(&["ask", "--model", "claude-opus-4-7", "x"])).unwrap();
        let args = ask_of(cli);
        assert_eq!(args.model, "claude-opus-4-7");
    }

    #[test]
    fn ask_max_tokens_override_parses() {
        let cli = Cli::parse(&argv(&["ask", "--max-tokens", "256", "x"])).unwrap();
        let args = ask_of(cli);
        assert_eq!(args.max_tokens, 256);
    }

    #[test]
    fn ask_max_tokens_invalid_int_errors() {
        assert!(matches!(
            Cli::parse(&argv(&["ask", "--max-tokens", "abc", "x"])),
            Err(ParseError::InvalidInt { opt, .. }) if opt == "--max-tokens"
        ));
    }

    #[test]
    fn ask_api_key_env_override_parses() {
        let cli = Cli::parse(&argv(&["ask", "--api-key-env", "MY_KEY", "x"])).unwrap();
        let args = ask_of(cli);
        assert_eq!(args.api_key_env, "MY_KEY");
    }

    #[test]
    fn ask_no_prompt_errors() {
        assert_eq!(Cli::parse(&argv(&["ask"])), Err(ParseError::MissingPrompt));
    }

    #[test]
    fn ask_stdin_and_positional_errors() {
        assert_eq!(
            Cli::parse(&argv(&["ask", "--stdin", "hello"])),
            Err(ParseError::PromptAndStdin)
        );
    }

    #[test]
    fn ask_unknown_long_flag_errors() {
        assert!(matches!(
            Cli::parse(&argv(&["ask", "--bogus", "x"])),
            Err(ParseError::UnknownOption(s)) if s == "--bogus"
        ));
    }

    #[test]
    fn ask_help_short_circuits() {
        assert_eq!(
            Cli::parse(&argv(&["ask", "--help"])),
            Err(ParseError::HelpRequested)
        );
    }

    #[test]
    fn audit_no_subcommand_errors() {
        assert_eq!(
            Cli::parse(&argv(&["audit"])),
            Err(ParseError::MissingAuditSubcommand)
        );
    }

    #[test]
    fn audit_unknown_subcommand_errors() {
        assert!(matches!(
            Cli::parse(&argv(&["audit", "list"])),
            Err(ParseError::UnknownAuditSubcommand(s)) if s == "list"
        ));
    }

    #[test]
    fn audit_verify_requires_audit_log() {
        assert_eq!(
            Cli::parse(&argv(&["audit", "verify", "--chain-key-file", "/k"])),
            Err(ParseError::MissingRequired("--audit-log"))
        );
    }

    #[test]
    fn audit_verify_requires_chain_key_file() {
        assert_eq!(
            Cli::parse(&argv(&["audit", "verify", "--audit-log", "/a"])),
            Err(ParseError::MissingRequired("--chain-key-file"))
        );
    }

    #[test]
    fn audit_verify_parses_both_paths() {
        let cli = Cli::parse(&argv(&[
            "audit",
            "verify",
            "--audit-log",
            "/a.jsonl",
            "--chain-key-file",
            "/k.bin",
        ]))
        .unwrap();
        let args = audit_verify_of(cli);
        assert_eq!(args.audit_log, PathBuf::from("/a.jsonl"));
        assert_eq!(args.chain_key_file, PathBuf::from("/k.bin"));
    }

    #[test]
    fn audit_verify_unknown_flag_errors() {
        assert!(matches!(
            Cli::parse(&argv(&["audit", "verify", "--bogus"])),
            Err(ParseError::UnknownOption(s)) if s == "--bogus"
        ));
    }

    #[test]
    fn audit_help_short_circuits() {
        assert_eq!(
            Cli::parse(&argv(&["audit", "--help"])),
            Err(ParseError::HelpRequested)
        );
    }

    #[test]
    fn session_no_subcommand_errors() {
        assert_eq!(
            Cli::parse(&argv(&["session"])),
            Err(ParseError::MissingSessionSubcommand)
        );
    }

    #[test]
    fn session_unknown_subcommand_errors() {
        assert!(matches!(
            Cli::parse(&argv(&["session", "stop"])),
            Err(ParseError::UnknownSessionSubcommand(s)) if s == "stop"
        ));
    }

    #[test]
    fn session_start_no_flags_uses_defaults() {
        let cli = Cli::parse(&argv(&["session", "start"])).unwrap();
        let args = session_start_of(cli);
        assert_eq!(args.model, "claude-sonnet-4-6");
        assert_eq!(args.max_tokens, 4096);
        assert_eq!(args.api_key_env, "ANTHROPIC_API_KEY");
        assert_eq!(args.system, None);
    }

    #[test]
    fn session_start_overrides_parse() {
        let cli = Cli::parse(&argv(&[
            "session",
            "start",
            "--model",
            "claude-opus-4-7",
            "--max-tokens",
            "1024",
            "--api-key-env",
            "MY_KEY",
            "--system",
            "be terse",
        ]))
        .unwrap();
        let args = session_start_of(cli);
        assert_eq!(args.model, "claude-opus-4-7");
        assert_eq!(args.max_tokens, 1024);
        assert_eq!(args.api_key_env, "MY_KEY");
        assert_eq!(args.system, Some("be terse".to_string()));
    }

    #[test]
    fn session_start_unknown_flag_errors() {
        assert!(matches!(
            Cli::parse(&argv(&["session", "start", "--bogus"])),
            Err(ParseError::UnknownOption(s)) if s == "--bogus"
        ));
    }

    #[test]
    fn session_help_short_circuits() {
        assert_eq!(
            Cli::parse(&argv(&["session", "--help"])),
            Err(ParseError::HelpRequested)
        );
    }
}
