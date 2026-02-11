# agent-workspace-template

A [cargo-generate](https://github.com/cargo-generate/cargo-generate) template for Rust projects optimized for AI agent workflows.

## Usage

```sh
cargo generate nikhiljha/agent-workspace-template
```

## Setup

After generating, run the setup script to install all tools and enable the pre-commit hook:

```sh
./setup.sh
```

## What you get

- Cargo workspace with a `lib` and `cli` crate
- Clippy pedantic + nursery lints
- `thiserror` (lib) / `anyhow` (cli) error handling
- `tracing` instrumentation pre-wired
- `tokio` async runtime
- `insta` snapshot tests
- `criterion` benchmarks
- `cargo nextest` as the test runner
- `cargo llvm-cov` coverage (100% enforced in CI)
- `cargo deny` for license/vulnerability auditing
- GitHub Actions CI
- `CLAUDE.md` for agent discoverability
