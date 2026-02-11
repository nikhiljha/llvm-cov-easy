#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Compact coverage gap analyzer for `llvm.coverage.json.export` data.
//!
//! Parses the JSON output from `cargo llvm-cov --json` and produces
//! ultra-compact, agent-friendly output showing exactly which lines,
//! regions, and branches lack coverage.

pub mod analysis;
pub mod format;
pub mod model;

use analysis::AnalysisResult;
use model::CoverageExport;

/// Parses coverage JSON from a string and analyzes it for coverage gaps.
///
/// This is the main entry point for the library. It deserializes the JSON,
/// runs coverage gap analysis, and returns the result.
///
/// # Errors
///
/// Returns an error if the JSON is malformed or the coverage data is empty.
pub fn analyze_json(json: &str) -> Result<AnalysisResult, Error> {
    let export: CoverageExport = serde_json::from_str(json)?;
    let result = analysis::analyze(&export)?;
    Ok(result)
}

/// Parses coverage JSON and returns formatted compact output.
///
/// Convenience function that combines parsing, analysis, and formatting.
///
/// # Errors
///
/// Returns an error if the JSON is malformed or the coverage data is empty.
pub fn analyze_and_format(json: &str) -> Result<String, Error> {
    let result = analyze_json(json)?;
    Ok(format::format_result(&result))
}

/// Errors that can occur in `llvm-cov-easy`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to parse the coverage JSON.
    #[error("failed to parse coverage JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Coverage analysis failed.
    #[error("{0}")]
    Analysis(#[from] analysis::AnalysisError),
}
