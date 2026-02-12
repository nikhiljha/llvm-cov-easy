#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use clap::{Parser, Subcommand};

/// Cargo wrapper for compact LLVM coverage output.
#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo", version, about)]
struct Cargo {
    /// Subcommand dispatched by cargo.
    #[command(subcommand)]
    command: CargoCommand,
}

/// Top-level cargo subcommand.
#[derive(Subcommand)]
enum CargoCommand {
    /// Compact coverage gap analyzer for LLVM coverage data.
    ///
    /// Takes the JSON output from `cargo llvm-cov --json` and produces
    /// ultra-compact, agent-friendly output showing exactly which lines,
    /// regions, and branches lack coverage.
    #[command(name = "llvm-cov-easy")]
    LlvmCovEasy {
        /// Subcommand to execute.
        #[command(subcommand)]
        command: Commands,
    },
}

/// Available subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Analyze coverage JSON and output compact coverage gaps.
    ///
    /// Reads a JSON file (or stdin if no path given) produced by
    /// `cargo llvm-cov --json` and outputs compact coverage gap information.
    Analyze {
        /// Path to the coverage JSON file. Reads from stdin if not provided.
        path: Option<PathBuf>,
    },
    /// Run `cargo llvm-cov run --json` and analyze the output.
    ///
    /// All trailing arguments are forwarded to `cargo llvm-cov run`.
    /// Use `+toolchain` (e.g. `+nightly`) as the first argument to select
    /// a Rust toolchain.
    Run {
        /// Arguments forwarded to `cargo llvm-cov run`.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run `cargo llvm-cov nextest --json` and analyze the output.
    ///
    /// All trailing arguments are forwarded to `cargo llvm-cov nextest`.
    /// Use `+toolchain` (e.g. `+nightly`) as the first argument to select
    /// a Rust toolchain.
    Nextest {
        /// Arguments forwarded to `cargo llvm-cov nextest`.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// Splits a `+toolchain` prefix from the user args, if present.
///
/// Returns the cargo args (e.g. `["cargo"]` or `["cargo", "+nightly"]`)
/// and the remaining args.
fn split_toolchain(user_args: &[String]) -> (Vec<&str>, &[String]) {
    if let Some(first) = user_args.first()
        && let Some(toolchain) = first.strip_prefix('+')
    {
        // Validate toolchain is non-empty
        if toolchain.is_empty() {
            return (vec!["cargo"], user_args);
        }
        return (vec!["cargo", &user_args[0]], &user_args[1..]);
    }
    (vec!["cargo"], user_args)
}

/// Builds the argument list for a `cargo llvm-cov` invocation.
fn build_cargo_llvm_cov_args<'a>(
    subcommand: &'a str,
    user_args: &'a [String],
) -> (Vec<&'a str>, Vec<&'a str>) {
    let (cargo_args, remaining) = split_toolchain(user_args);
    let mut args = vec!["llvm-cov", subcommand, "--json"];
    args.extend(remaining.iter().map(String::as_str));
    (cargo_args, args)
}

/// Runs `cargo llvm-cov <subcommand> --json [args...]` and returns its stdout.
///
/// Stderr is inherited so users see compilation and test progress.
///
/// COVERAGE: This function spawns an external process (`cargo llvm-cov`)
/// which requires the full toolchain and is tested via E2E tests.
#[cfg_attr(coverage_nightly, coverage(off))]
fn run_cargo_llvm_cov(subcommand: &str, user_args: &[String]) -> anyhow::Result<String> {
    let (cargo_args, llvm_cov_args) = build_cargo_llvm_cov_args(subcommand, user_args);

    let output = Command::new(cargo_args[0])
        .args(&cargo_args[1..])
        .args(&llvm_cov_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "{} {} exited with status {}",
            cargo_args.join(" "),
            llvm_cov_args.join(" "),
            output.status
        );
    }

    Ok(String::from_utf8(output.stdout)?)
}

/// COVERAGE: main is the thin entry point; logic is tested via the library crate.
#[cfg_attr(coverage_nightly, coverage(off))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let Cargo {
        command: CargoCommand::LlvmCovEasy { command },
    } = Cargo::parse();

    let json = match command {
        Commands::Analyze { path } => read_input(path)?,
        Commands::Run { args } => run_cargo_llvm_cov("run", &args)?,
        Commands::Nextest { args } => run_cargo_llvm_cov("nextest", &args)?,
    };

    let mut result = llvm_cov_easy::analyze_json(&json)?;
    if let Ok(cwd) = std::env::current_dir() {
        result.relativize_paths(&cwd);
    }
    let output = llvm_cov_easy::format::format_result(&result);
    print!("{output}");

    Ok(())
}

/// Reads JSON input from a file or stdin.
///
/// COVERAGE: This function involves I/O (stdin/file reads) that is tested
/// via integration tests, not unit tests.
#[cfg_attr(coverage_nightly, coverage(off))]
fn read_input(path: Option<PathBuf>) -> anyhow::Result<String> {
    if let Some(p) = path {
        Ok(std::fs::read_to_string(&p)?)
    } else {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_toolchain_with_nightly() {
        let args = vec!["+nightly".to_string(), "--workspace".to_string()];
        let (cargo, rest) = split_toolchain(&args);
        assert_eq!(cargo, vec!["cargo", "+nightly"]);
        assert_eq!(rest, &[String::from("--workspace")]);
    }

    #[test]
    fn split_toolchain_without_toolchain() {
        let args = vec!["--workspace".to_string()];
        let (cargo, rest) = split_toolchain(&args);
        assert_eq!(cargo, vec!["cargo"]);
        assert_eq!(rest, &args[..]);
    }

    #[test]
    fn split_toolchain_empty_args() {
        let args: Vec<String> = vec![];
        let (cargo, rest) = split_toolchain(&args);
        assert_eq!(cargo, vec!["cargo"]);
        assert!(rest.is_empty());
    }

    #[test]
    fn split_toolchain_bare_plus() {
        let args = vec!["+".to_string(), "--workspace".to_string()];
        let (cargo, rest) = split_toolchain(&args);
        assert_eq!(cargo, vec!["cargo"]);
        assert_eq!(rest, &args[..]);
    }

    #[test]
    fn build_args_no_user_args() {
        let user_args: Vec<String> = vec![];
        let (cargo, llvm_args) = build_cargo_llvm_cov_args("nextest", &user_args);
        assert_eq!(cargo, vec!["cargo"]);
        assert_eq!(llvm_args, vec!["llvm-cov", "nextest", "--json"]);
    }

    #[test]
    fn build_args_with_user_args() {
        let user_args = vec![
            "--workspace".to_string(),
            "--branch".to_string(),
            "--no-fail-fast".to_string(),
        ];
        let (cargo, llvm_args) = build_cargo_llvm_cov_args("nextest", &user_args);
        assert_eq!(cargo, vec!["cargo"]);
        assert_eq!(
            llvm_args,
            vec![
                "llvm-cov",
                "nextest",
                "--json",
                "--workspace",
                "--branch",
                "--no-fail-fast"
            ]
        );
    }

    #[test]
    fn build_args_run_subcommand() {
        let user_args = vec!["--".to_string(), "--help".to_string()];
        let (cargo, llvm_args) = build_cargo_llvm_cov_args("run", &user_args);
        assert_eq!(cargo, vec!["cargo"]);
        assert_eq!(llvm_args, vec!["llvm-cov", "run", "--json", "--", "--help"]);
    }

    #[test]
    fn build_args_with_toolchain() {
        let user_args = vec![
            "+nightly".to_string(),
            "--workspace".to_string(),
            "--branch".to_string(),
        ];
        let (cargo, llvm_args) = build_cargo_llvm_cov_args("nextest", &user_args);
        assert_eq!(cargo, vec!["cargo", "+nightly"]);
        assert_eq!(
            llvm_args,
            vec!["llvm-cov", "nextest", "--json", "--workspace", "--branch"]
        );
    }
}
