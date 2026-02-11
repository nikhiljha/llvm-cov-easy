//! Deserialization types for the `llvm.coverage.json.export` format.
//!
//! Supports versions `2.0.1` and `3.1.0` of the export format.

use serde::Deserialize;

/// Top-level coverage export structure.
#[derive(Debug, Deserialize)]
pub struct CoverageExport {
    /// Coverage data entries (typically one element).
    pub data: Vec<ExportData>,
    /// Format identifier, expected to be `"llvm.coverage.json.export"`.
    #[serde(rename = "type")]
    pub export_type: String,
    /// Format version (e.g., `"2.0.1"` or `"3.1.0"`).
    pub version: String,
}

/// A single coverage data entry containing files, functions, and totals.
#[derive(Debug, Deserialize)]
pub struct ExportData {
    /// Per-file coverage data.
    pub files: Vec<FileData>,
    /// Per-function coverage data.
    #[serde(default)]
    pub functions: Vec<FunctionData>,
    /// Aggregate coverage totals across all files.
    pub totals: Summary,
}

/// Coverage data for a single source file.
#[derive(Debug, Deserialize)]
pub struct FileData {
    /// File path as it appears in the coverage data.
    pub filename: String,
    /// Coverage segments (line, col, count, `has_count`, `is_region_entry`,
    /// `is_gap_region`).
    #[serde(default)]
    pub segments: Vec<Segment>,
    /// Branch coverage entries (present when `--branch` was used).
    #[serde(default)]
    pub branches: Vec<Branch>,
    /// File-level coverage summary.
    pub summary: Summary,
}

/// A coverage segment representing a point where coverage state changes.
///
/// Segments form a state machine: each segment sets the execution count
/// from its position until the next segment.
#[derive(Debug, Clone)]
pub struct Segment {
    /// Line number (1-based).
    pub line: u64,
    /// Column number (1-based).
    pub col: u64,
    /// Execution count at this point.
    pub count: u64,
    /// Whether this segment has a meaningful count.
    pub has_count: bool,
    /// Whether this segment starts a new region.
    pub is_region_entry: bool,
    /// Whether this is a gap region (inserted for non-code areas).
    pub is_gap_region: bool,
}

impl<'de> Deserialize<'de> for Segment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr: (u64, u64, u64, bool, bool, bool) = Deserialize::deserialize(deserializer)?;
        Ok(Self {
            line: arr.0,
            col: arr.1,
            count: arr.2,
            has_count: arr.3,
            is_region_entry: arr.4,
            is_gap_region: arr.5,
        })
    }
}

/// A branch coverage entry with true/false execution counts.
#[derive(Debug, Clone)]
pub struct Branch {
    /// Starting line number.
    pub line_start: u64,
    /// Starting column number.
    pub col_start: u64,
    /// Ending line number.
    pub line_end: u64,
    /// Ending column number.
    pub col_end: u64,
    /// Number of times the true branch was taken.
    pub true_count: u64,
    /// Number of times the false branch was taken.
    pub false_count: u64,
}

impl<'de> Deserialize<'de> for Branch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Format: [LineStart, ColStart, LineEnd, ColEnd, TrueCount, FalseCount,
        //          FileID, ExpandedFileID, Kind]
        let arr: (u64, u64, u64, u64, u64, u64, u64, u64, u64) =
            Deserialize::deserialize(deserializer)?;
        Ok(Self {
            line_start: arr.0,
            col_start: arr.1,
            line_end: arr.2,
            col_end: arr.3,
            true_count: arr.4,
            false_count: arr.5,
        })
    }
}

/// Per-function coverage data.
#[derive(Debug, Deserialize)]
pub struct FunctionData {
    /// Mangled function name.
    pub name: String,
    /// Execution count.
    pub count: u64,
    /// Source files this function spans.
    pub filenames: Vec<String>,
    /// Region coverage data.
    pub regions: Vec<Region>,
    /// Branch coverage data.
    #[serde(default)]
    pub branches: Vec<Branch>,
}

/// A coverage region within a function.
#[derive(Debug, Clone)]
pub struct Region {
    /// Starting line number.
    pub line_start: u64,
    /// Starting column number.
    pub col_start: u64,
    /// Ending line number.
    pub line_end: u64,
    /// Ending column number.
    pub col_end: u64,
    /// Number of times this region was executed.
    pub execution_count: u64,
    /// File ID within the function's file list.
    pub file_id: u64,
    /// Expanded file ID.
    pub expanded_file_id: u64,
    /// Region kind.
    pub kind: u64,
}

impl<'de> Deserialize<'de> for Region {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr: (u64, u64, u64, u64, u64, u64, u64, u64) = Deserialize::deserialize(deserializer)?;
        Ok(Self {
            line_start: arr.0,
            col_start: arr.1,
            line_end: arr.2,
            col_end: arr.3,
            execution_count: arr.4,
            file_id: arr.5,
            expanded_file_id: arr.6,
            kind: arr.7,
        })
    }
}

/// Coverage summary with counts for different coverage metrics.
#[derive(Debug, Deserialize)]
pub struct Summary {
    /// Branch coverage counts.
    #[serde(default)]
    pub branches: Option<CoverageCounts>,
    /// Function coverage counts.
    #[serde(default)]
    pub functions: Option<CoverageCounts>,
    /// Instantiation coverage counts.
    #[serde(default)]
    pub instantiations: Option<CoverageCounts>,
    /// Line coverage counts.
    #[serde(default)]
    pub lines: Option<CoverageCounts>,
    /// Region coverage counts.
    #[serde(default)]
    pub regions: Option<CoverageCounts>,
}

/// Count and percentage for a single coverage metric.
#[derive(Debug, Deserialize)]
pub struct CoverageCounts {
    /// Total count.
    pub count: u64,
    /// Number of covered items.
    pub covered: u64,
    /// Coverage percentage.
    pub percent: f64,
}
