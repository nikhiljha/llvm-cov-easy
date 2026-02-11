//! Coverage gap analysis.
//!
//! Analyzes [`CoverageExport`] data to find uncovered lines, regions, and
//! branches, then collapses consecutive uncovered lines into ranges.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{CoverageExport, FileData, Segment};

/// A coverage gap found during analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageGap {
    /// One or more consecutive fully-uncovered lines.
    UncoveredLines {
        /// First uncovered line (1-based).
        start_line: u64,
        /// Last uncovered line (1-based, inclusive).
        end_line: u64,
    },
    /// An uncovered region on a partially-covered line.
    UncoveredRegion {
        /// Start line.
        line_start: u64,
        /// Start column.
        col_start: u64,
        /// End line.
        line_end: u64,
        /// End column.
        col_end: u64,
    },
    /// A branch where one direction was never taken.
    UncoveredBranch {
        /// Line where the branch occurs.
        line: u64,
        /// Column where the branch occurs.
        col: u64,
        /// Number of times the true branch was taken.
        true_count: u64,
        /// Number of times the false branch was taken.
        false_count: u64,
    },
}

/// Per-file coverage gap results.
#[derive(Debug, Clone)]
pub struct FileGaps {
    /// File path as it appears in the coverage data.
    pub filename: String,
    /// Coverage gaps found in this file, sorted by location.
    pub gaps: Vec<CoverageGap>,
}

/// Summary coverage percentages.
#[derive(Debug, Clone)]
pub struct CoverageSummary {
    /// Line coverage percentage (0.0-100.0).
    pub lines_percent: f64,
    /// Region coverage percentage (0.0-100.0).
    pub regions_percent: f64,
    /// Branch coverage percentage (0.0-100.0), if branch data is present.
    pub branches_percent: Option<f64>,
    /// Function coverage percentage (0.0-100.0).
    pub functions_percent: f64,
}

/// Complete analysis result.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Per-file coverage gaps (only files with gaps are included).
    pub files: Vec<FileGaps>,
    /// Overall coverage summary.
    pub summary: CoverageSummary,
}

/// Analyzes a coverage export and returns all coverage gaps.
///
/// # Errors
///
/// Returns an error if the coverage data is empty.
pub fn analyze(export: &CoverageExport) -> Result<AnalysisResult, AnalysisError> {
    let data = export.data.first().ok_or(AnalysisError::EmptyData)?;

    let mut files = Vec::new();
    for file in &data.files {
        let gaps = analyze_file(file);
        if !gaps.is_empty() {
            files.push(FileGaps {
                filename: file.filename.clone(),
                gaps,
            });
        }
    }

    let totals = &data.totals;
    let branches_percent = totals
        .branches
        .as_ref()
        .and_then(|b| if b.count > 0 { Some(b.percent) } else { None });

    let summary = CoverageSummary {
        lines_percent: totals.lines.as_ref().map_or(0.0, |l| l.percent),
        regions_percent: totals.regions.as_ref().map_or(0.0, |r| r.percent),
        branches_percent,
        functions_percent: totals.functions.as_ref().map_or(0.0, |f| f.percent),
    };

    Ok(AnalysisResult { files, summary })
}

/// Errors that can occur during analysis.
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    /// The coverage data array is empty.
    #[error("coverage data is empty (no data entries)")]
    EmptyData,
}

/// Analyzes a single file's coverage data and returns its gaps.
fn analyze_file(file: &FileData) -> Vec<CoverageGap> {
    let mut gaps = Vec::new();

    if !file.segments.is_empty() {
        let (uncovered_lines, uncovered_regions) = analyze_segments(&file.segments);
        gaps.extend(uncovered_lines);
        gaps.extend(uncovered_regions);
    }

    for branch in &file.branches {
        if branch.true_count == 0 || branch.false_count == 0 {
            gaps.push(CoverageGap::UncoveredBranch {
                line: branch.line_start,
                col: branch.col_start,
                true_count: branch.true_count,
                false_count: branch.false_count,
            });
        }
    }

    gaps
}

/// Represents a region entry from a segment with its span derived from
/// the next segment.
struct RegionSpan {
    line_start: u64,
    col_start: u64,
    line_end: u64,
    col_end: u64,
}

/// Analyzes segments to find uncovered lines and sub-line regions.
///
/// Returns `(uncovered_line_gaps, uncovered_region_gaps)`.
fn analyze_segments(segments: &[Segment]) -> (Vec<CoverageGap>, Vec<CoverageGap>) {
    // Build per-line coverage: track the max count seen on each line.
    // Also collect region entries with their spans for sub-line analysis.
    let mut line_max_count: BTreeMap<u64, u64> = BTreeMap::new();
    let mut lines_with_coverage: BTreeSet<u64> = BTreeSet::new();
    let mut region_spans: Vec<RegionSpan> = Vec::new();

    for i in 0..segments.len() {
        let seg = &segments[i];
        if !seg.has_count {
            continue;
        }

        // Determine span end from the next segment (or same point if last).
        let (end_line, end_col) = if i + 1 < segments.len() {
            (segments[i + 1].line, segments[i + 1].col)
        } else {
            (seg.line, seg.col)
        };

        // Record counts for all lines this segment spans.
        for line in seg.line..=end_line {
            let entry = line_max_count.entry(line).or_insert(0);
            *entry = (*entry).max(seg.count);
            if seg.count > 0 {
                lines_with_coverage.insert(line);
            }
        }

        // Collect region entry spans for sub-line analysis.
        if seg.is_region_entry && seg.count == 0 {
            region_spans.push(RegionSpan {
                line_start: seg.line,
                col_start: seg.col,
                line_end: end_line,
                col_end: end_col,
            });
        }
    }

    // Find fully uncovered lines (max count == 0, and line was tracked).
    let uncovered_lines: BTreeSet<u64> = line_max_count
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(line, _)| *line)
        .collect();

    let line_gaps = collapse_lines(&uncovered_lines);

    // Find uncovered regions on partially-covered lines.
    // A region is "sub-line" if its start line has other coverage.
    let region_gaps: Vec<CoverageGap> = region_spans
        .into_iter()
        .filter(|r| {
            // Only show as REGION if the start line is NOT fully uncovered
            // (otherwise it's already shown as UNCOVERED).
            !uncovered_lines.contains(&r.line_start) && lines_with_coverage.contains(&r.line_start)
        })
        .map(|r| CoverageGap::UncoveredRegion {
            line_start: r.line_start,
            col_start: r.col_start,
            line_end: r.line_end,
            col_end: r.col_end,
        })
        .collect();

    (line_gaps, region_gaps)
}

/// Collapses a set of line numbers into consecutive ranges.
fn collapse_lines(lines: &BTreeSet<u64>) -> Vec<CoverageGap> {
    let mut gaps = Vec::new();
    let mut iter = lines.iter().copied();

    let Some(first) = iter.next() else {
        return gaps;
    };

    let mut start = first;
    let mut end = first;

    for line in iter {
        if line == end + 1 {
            end = line;
        } else {
            gaps.push(CoverageGap::UncoveredLines {
                start_line: start,
                end_line: end,
            });
            start = line;
            end = line;
        }
    }

    gaps.push(CoverageGap::UncoveredLines {
        start_line: start,
        end_line: end,
    });

    gaps
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_collapse_lines_empty() {
        let lines = BTreeSet::new();
        let gaps = collapse_lines(&lines);
        assert!(gaps.is_empty());
    }

    #[test]
    fn test_collapse_lines_single() {
        let lines = BTreeSet::from([5]);
        let gaps = collapse_lines(&lines);
        assert_eq!(gaps.len(), 1);
        assert_eq!(
            gaps[0],
            CoverageGap::UncoveredLines {
                start_line: 5,
                end_line: 5,
            }
        );
    }

    #[test]
    fn test_analyze_segments_last_segment_has_count() {
        // Edge case: last segment has has_count=true and is the final segment,
        // so the fallback (seg.line, seg.col) path is exercised.
        // Segment 0 covers line 1 with count=1, ending at segment 1 (line 2).
        // Segment 1 (no has_count) is skipped.
        // Segment 2 is the last segment with has_count=true on a fresh line.
        let segments = vec![
            Segment {
                line: 1,
                col: 1,
                count: 1,
                has_count: true,
                is_region_entry: true,
                is_gap_region: false,
            },
            Segment {
                line: 2,
                col: 1,
                count: 0,
                has_count: false,
                is_region_entry: false,
                is_gap_region: false,
            },
            Segment {
                line: 5,
                col: 1,
                count: 0,
                has_count: true,
                is_region_entry: true,
                is_gap_region: false,
            },
        ];
        let (line_gaps, region_gaps) = analyze_segments(&segments);
        // Line 5 should be uncovered (last segment fallback path).
        assert_eq!(line_gaps.len(), 1);
        assert_eq!(
            line_gaps[0],
            CoverageGap::UncoveredLines {
                start_line: 5,
                end_line: 5,
            }
        );
        // The region at line 5 is fully uncovered, so no sub-line REGION.
        assert!(region_gaps.is_empty());
    }

    #[test]
    fn test_collapse_lines_consecutive() {
        let lines = BTreeSet::from([3, 4, 5, 10, 11, 15]);
        let gaps = collapse_lines(&lines);
        assert_eq!(gaps.len(), 3);
        assert_eq!(
            gaps[0],
            CoverageGap::UncoveredLines {
                start_line: 3,
                end_line: 5,
            }
        );
        assert_eq!(
            gaps[1],
            CoverageGap::UncoveredLines {
                start_line: 10,
                end_line: 11,
            }
        );
        assert_eq!(
            gaps[2],
            CoverageGap::UncoveredLines {
                start_line: 15,
                end_line: 15,
            }
        );
    }
}
