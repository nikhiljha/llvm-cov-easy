# llvm-cov-quick

## Motivation

AI coding agents need to quickly understand where test coverage is missing. The raw JSON output from `cargo llvm-cov --json` is huge and full of data irrelevant to the task of "find what's uncovered." Existing options like `--show-missing-lines` only show line-level info and still require grep/parsing. Agents waste tokens parsing verbose output when they just need: "file X, line Y is uncovered" or "file X, line Y, the false branch was never taken."

`llvm-cov-quick` takes the JSON output from `cargo llvm-cov --json` (the `llvm.coverage.json.export` format) and produces ultra-compact, agent-friendly output showing exactly which lines, regions, and branches lack coverage.

## Architecture

This is a Cargo workspace with two crates:
- `lib/` (`llvm-cov-quick`): Core logic — JSON deserialization, analysis, compact output formatting. Uses `thiserror` for errors and `tracing` for instrumentation.
- `cli/` (`llvm-cov-quick-cli`): Binary entry point with subcommand dispatch. Uses `tokio`, `anyhow`, and `tracing-subscriber`.

## Input Format

The tool consumes the JSON produced by `cargo llvm-cov --json` (which internally calls `llvm-cov export -format=text`). This is the `llvm.coverage.json.export` format with version `2.0.1` or `3.1.0`. The JSON schema (see `~/Development/github.com/taiki-e/cargo-llvm-cov/src/json.rs` for reference types) contains:

- **Files**: Each file has segments (line, col, count, has_count, is_region_entry, is_gap_region) and a summary with counts for branches, functions, instantiations, lines, and regions.
- **Functions**: Each function has regions (LineStart, ColumnStart, LineEnd, ColumnEnd, ExecutionCount, FileID, ExpandedFileID, Kind) and branch data.
- **Branches**: Present when `--branch` flag was used with `cargo llvm-cov`. Each branch entry in the file's `branches` array contains line/column span and true/false execution counts.
- **Totals**: Aggregate coverage counts across all files.

The tool must handle both cases: branches present (when `--branch` was used) and branches absent.

## Subcommands

### `analyze` (the initial and primary subcommand)

Takes a path to a JSON file (or reads from stdin if no path given) and outputs compact coverage gap information.

**Output format:**

```
src/lib.rs:7 UNCOVERED
src/lib.rs:8-9 UNCOVERED
src/lib.rs:42:3-42:18 REGION hits:0
src/lib.rs:50:5 BRANCH true:5 false:0
Lines: 92.3% | Regions: 88.1% | Branches: 75.0% | Functions: 100.0%
```

Design principles for output:
- One line per coverage gap (or a collapsed range for consecutive uncovered lines)
- Only show what's *missing* — never show covered code
- Collapse consecutive uncovered lines into ranges (e.g., `7-9` instead of three separate lines)
- For regions: show sub-line precision only when there are multiple regions on a line (otherwise just show line-level)
- For branches: show the true/false execution counts so the agent knows which case is missing
- File paths as they appear in the coverage data (relative paths)
- No color codes, no decorations, no headers unless a summary is requested
- Summary line at the end with total coverage percentages for lines, regions, branches, and functions

### Reserved: base command (future)

The base command (no subcommand) is reserved for a future wrapper that will invoke `cargo llvm-cov` itself and then analyze the output. Not in scope now.

## Reference Material

- cargo-llvm-cov source: `~/Development/github.com/taiki-e/cargo-llvm-cov` (especially `src/json.rs` for the JSON schema types)
- LLVM docs: https://llvm.org/docs/CommandGuide/llvm-cov.html (export command section)
- Test fixtures with real JSON: `~/Development/github.com/taiki-e/cargo-llvm-cov/tests/fixtures/`

## Testing Strategy

- Use real JSON fixtures from the cargo-llvm-cov test suite (copy relevant ones into this project)
- Snapshot tests with `insta` for the compact output format
- Unit tests for the JSON deserialization, the analysis/collapsing logic, and the formatting
- Edge cases: empty coverage data, no branches, all covered (should produce minimal/no output), malformed JSON
