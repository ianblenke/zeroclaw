# Testing Infrastructure Specification

## Purpose
Define the requirements for test coverage measurement, reporting, and enforcement across the ZeroClaw codebase.

## Scope
- Files: `tarpaulin.toml`, `dev/coverage.sh`
- Risk tier: LOW

## Requirements

### REQ-COV-001: Coverage measurement MUST be available via cargo-tarpaulin
The project MUST provide a `tarpaulin.toml` configuration file that enables `cargo tarpaulin` to measure line-level code coverage for all Rust source files.

#### Scenario: Tarpaulin config exists and is valid
- WHEN `cargo tarpaulin --config tarpaulin.toml` is invoked
- THEN the command completes without configuration errors and produces coverage output

#### Scenario: Test code is excluded from coverage measurement
- WHEN coverage is measured
- THEN files under `tests/`, `benches/`, and `src/bin/` are excluded from the coverage denominator

### REQ-COV-002: Per-module coverage breakdown MUST be available
The project MUST provide a `dev/coverage.sh` script that reports coverage per top-level module (e.g., `src/providers/`, `src/channels/`, `src/tools/`).

#### Scenario: Full coverage report
- WHEN `./dev/coverage.sh` is invoked with no arguments
- THEN it produces a per-module coverage summary showing module name, covered lines, total lines, and percentage

#### Scenario: Single module coverage
- WHEN `./dev/coverage.sh <module>` is invoked with a module name
- THEN it reports coverage for only the files under `src/<module>/`

### REQ-COV-003: Coverage reports MUST be generated in HTML and JSON format
The tarpaulin configuration MUST produce both HTML (human-readable) and JSON (machine-parseable) reports under `target/tarpaulin/`.

#### Scenario: Report output format
- WHEN coverage measurement completes
- THEN `target/tarpaulin/tarpaulin-report.html` and `target/tarpaulin/tarpaulin-report.json` exist

### REQ-COV-004: Coverage script MUST auto-install tarpaulin if missing
The `dev/coverage.sh` script MUST detect if `cargo-tarpaulin` is not installed and install it automatically before proceeding.

#### Scenario: First-time run without tarpaulin installed
- WHEN `./dev/coverage.sh` is invoked and `cargo tarpaulin --version` fails
- THEN the script runs `cargo install cargo-tarpaulin` before measuring coverage

## Mock Strategy
Not applicable — this spec covers tooling infrastructure, not testable Rust code.

## Coverage Notes
- This spec is the bootstrap for all subsequent coverage-tracked specs
- Baseline coverage will be recorded after first successful tarpaulin run
