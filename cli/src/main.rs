#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use clap::{Parser, Subcommand};

/// Compact coverage gap analyzer for LLVM coverage data.
///
/// Takes the JSON output from `cargo llvm-cov --json` and produces
/// ultra-compact, agent-friendly output showing exactly which lines,
/// regions, and branches lack coverage.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    command: Commands,
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
    Run {
        /// Arguments forwarded to `cargo llvm-cov run`.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run `cargo llvm-cov nextest --json` and analyze the output.
    ///
    /// All trailing arguments are forwarded to `cargo llvm-cov nextest`.
    Nextest {
        /// Arguments forwarded to `cargo llvm-cov nextest`.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// Builds the argument list for a `cargo llvm-cov` invocation.
fn build_cargo_llvm_cov_args<'a>(subcommand: &'a str, user_args: &'a [String]) -> Vec<&'a str> {
    let mut args = vec!["llvm-cov", subcommand, "--json"];
    args.extend(user_args.iter().map(String::as_str));
    args
}

/// Runs `cargo llvm-cov <subcommand> --json [args...]` and returns its stdout.
///
/// Stderr is inherited so users see compilation and test progress.
///
/// COVERAGE: This function spawns an external process (`cargo llvm-cov`)
/// which requires the full toolchain and is tested via E2E tests.
#[cfg_attr(coverage_nightly, coverage(off))]
fn run_cargo_llvm_cov(subcommand: &str, user_args: &[String]) -> anyhow::Result<String> {
    let args = build_cargo_llvm_cov_args(subcommand, user_args);

    let output = Command::new("cargo")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "cargo {} exited with status {}",
            args.join(" "),
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

    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { path } => {
            let json = read_input(path)?;
            let output = llvm_cov_easy::analyze_and_format(&json)?;
            print!("{output}");
        }
        Commands::Run { args } => {
            let json = run_cargo_llvm_cov("run", &args)?;
            let output = llvm_cov_easy::analyze_and_format(&json)?;
            print!("{output}");
        }
        Commands::Nextest { args } => {
            let json = run_cargo_llvm_cov("nextest", &args)?;
            let output = llvm_cov_easy::analyze_and_format(&json)?;
            print!("{output}");
        }
    }

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
    fn build_args_no_user_args() {
        let user_args: Vec<String> = vec![];
        let result = build_cargo_llvm_cov_args("nextest", &user_args);
        assert_eq!(result, vec!["llvm-cov", "nextest", "--json"]);
    }

    #[test]
    fn build_args_with_user_args() {
        let user_args = vec![
            "--workspace".to_string(),
            "--branch".to_string(),
            "--no-fail-fast".to_string(),
        ];
        let result = build_cargo_llvm_cov_args("nextest", &user_args);
        assert_eq!(
            result,
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
        let result = build_cargo_llvm_cov_args("run", &user_args);
        assert_eq!(result, vec!["llvm-cov", "run", "--json", "--", "--help"]);
    }
}
