//! Compact output formatting for coverage gaps.
//!
//! Formats [`AnalysisResult`] into agent-friendly text output.

use std::fmt::Write;

use crate::analysis::{AnalysisResult, CoverageGap, CoverageSummary};

/// Formats an analysis result as compact, agent-friendly text.
///
/// Output format:
/// ```text
/// src/lib.rs:7 UNCOVERED
/// src/lib.rs:8-9 UNCOVERED
/// src/lib.rs:42:3-42:18 REGION hits:0
/// src/lib.rs:50:5 BRANCH true:5 false:0
/// Lines: 92.3% | Regions: 88.1% | Branches: 75.0% | Functions: 100.0%
/// ```
#[must_use]
pub fn format_result(result: &AnalysisResult) -> String {
    let mut output = String::new();

    for file in &result.files {
        for gap in &file.gaps {
            format_gap(&mut output, &file.filename, gap);
        }
    }

    format_summary(&mut output, &result.summary);
    output
}

/// Formats a single coverage gap into the output buffer.
fn format_gap(output: &mut String, filename: &str, gap: &CoverageGap) {
    match gap {
        CoverageGap::UncoveredLines {
            start_line,
            end_line,
        } => {
            if start_line == end_line {
                writeln!(output, "{filename}:{start_line} UNCOVERED")
            } else {
                writeln!(output, "{filename}:{start_line}-{end_line} UNCOVERED")
            }
        }
        CoverageGap::UncoveredRegion {
            line_start,
            col_start,
            line_end,
            col_end,
        } => writeln!(
            output,
            "{filename}:{line_start}:{col_start}-{line_end}:{col_end} REGION hits:0"
        ),
        CoverageGap::UncoveredBranch {
            line,
            col,
            true_count,
            false_count,
        } => writeln!(
            output,
            "{filename}:{line}:{col} BRANCH true:{true_count} false:{false_count}"
        ),
    }
    // writeln to a String is infallible.
    .unwrap();
}

/// Formats the summary line.
fn format_summary(output: &mut String, summary: &CoverageSummary) {
    let lines = format_percent(summary.lines_percent);
    let regions = format_percent(summary.regions_percent);
    let functions = format_percent(summary.functions_percent);

    match summary.branches_percent {
        Some(bp) => {
            let branches = format_percent(bp);
            write!(
                output,
                "Lines: {lines} | Regions: {regions} | Branches: {branches} | Functions: {functions}"
            )
        }
        None => write!(
            output,
            "Lines: {lines} | Regions: {regions} | Functions: {functions}"
        ),
    }
    // write to a String is infallible.
    .unwrap();
}

/// Formats a percentage with one decimal place, dropping trailing `.0`.
fn format_percent(value: f64) -> String {
    let formatted = format!("{value:.1}%");
    // Clean up "100.0%" -> "100.0%" (keep it consistent, always one decimal)
    formatted
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::analysis::FileGaps;

    #[test]
    fn test_format_single_uncovered_line() {
        let result = AnalysisResult {
            files: vec![FileGaps {
                filename: "src/lib.rs".to_string(),
                gaps: vec![CoverageGap::UncoveredLines {
                    start_line: 7,
                    end_line: 7,
                }],
            }],
            summary: CoverageSummary {
                lines_percent: 92.3,
                regions_percent: 88.1,
                branches_percent: None,
                functions_percent: 100.0,
            },
        };

        let output = format_result(&result);
        assert!(output.contains("src/lib.rs:7 UNCOVERED"));
    }

    #[test]
    fn test_format_uncovered_range() {
        let result = AnalysisResult {
            files: vec![FileGaps {
                filename: "src/lib.rs".to_string(),
                gaps: vec![CoverageGap::UncoveredLines {
                    start_line: 8,
                    end_line: 10,
                }],
            }],
            summary: CoverageSummary {
                lines_percent: 90.0,
                regions_percent: 85.0,
                branches_percent: None,
                functions_percent: 100.0,
            },
        };

        let output = format_result(&result);
        assert!(output.contains("src/lib.rs:8-10 UNCOVERED"));
    }

    #[test]
    fn test_format_branch_gap() {
        let result = AnalysisResult {
            files: vec![FileGaps {
                filename: "src/lib.rs".to_string(),
                gaps: vec![CoverageGap::UncoveredBranch {
                    line: 50,
                    col: 5,
                    true_count: 5,
                    false_count: 0,
                }],
            }],
            summary: CoverageSummary {
                lines_percent: 92.3,
                regions_percent: 88.1,
                branches_percent: Some(75.0),
                functions_percent: 100.0,
            },
        };

        let output = format_result(&result);
        assert!(output.contains("src/lib.rs:50:5 BRANCH true:5 false:0"));
        assert!(output.contains("Branches: 75.0%"));
    }

    #[test]
    fn test_format_summary_without_branches() {
        let summary = CoverageSummary {
            lines_percent: 92.3,
            regions_percent: 88.1,
            branches_percent: None,
            functions_percent: 100.0,
        };
        let mut output = String::new();
        format_summary(&mut output, &summary);
        assert_eq!(output, "Lines: 92.3% | Regions: 88.1% | Functions: 100.0%");
    }

    #[test]
    fn test_format_summary_with_branches() {
        let summary = CoverageSummary {
            lines_percent: 92.3,
            regions_percent: 88.1,
            branches_percent: Some(75.0),
            functions_percent: 100.0,
        };
        let mut output = String::new();
        format_summary(&mut output, &summary);
        assert_eq!(
            output,
            "Lines: 92.3% | Regions: 88.1% | Branches: 75.0% | Functions: 100.0%"
        );
    }
}
