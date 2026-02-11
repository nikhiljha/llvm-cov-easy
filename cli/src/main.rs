#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::io::Read;
use std::path::PathBuf;

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
