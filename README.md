# llvm-cov-easy

Compact coverage gap analyzer for LLVM coverage data. Takes the verbose JSON output from [`cargo llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) and produces ultra-compact output showing exactly which lines, regions, and branches lack coverage.

Designed for AI coding agents that need to quickly understand where test coverage is missing without wasting tokens on verbose output.

## Installation

```bash
cargo install --git https://github.com/nikhiljha/llvm-cov-easy
```

Requires [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) for the `run` and `nextest` subcommands. For `nextest`, you also need [`cargo-nextest`](https://nexte.st/).

## Usage

### One-step (recommended)

Run tests and get compact coverage output in a single command:

```bash
# Using nextest
cargo llvm-cov-easy nextest --workspace --branch

# Using cargo test
cargo llvm-cov-easy run
```

All trailing arguments are forwarded to `cargo llvm-cov`:

```bash
cargo llvm-cov-easy nextest --workspace --branch --no-fail-fast
cargo llvm-cov-easy run -- --test-threads=1
```

### Two-step

If you already have JSON coverage output, pipe it through `analyze`:

```bash
cargo llvm-cov nextest --json --workspace --branch | cargo llvm-cov-easy analyze

# Or from a file
cargo llvm-cov-easy analyze coverage.json
```

## Output format

```
src/lib.rs:7 UNCOVERED
src/lib.rs:8-9 UNCOVERED
src/lib.rs:42:3-42:18 REGION hits:0
src/lib.rs:50:5 BRANCH true:5 false:0
Lines: 92.3% | Regions: 88.1% | Branches: 75.0% | Functions: 100.0%
```

- One line per coverage gap, consecutive uncovered lines collapsed into ranges
- Only shows what's missing -- covered code is never shown
- Sub-line precision for regions only when there are multiple regions on a line
- Branch entries show true/false execution counts so you know which case is missing
- Summary line with total coverage percentages

## License

MIT
