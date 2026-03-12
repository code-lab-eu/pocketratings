---
name: backend-quality-control
description: Run format, lint, and test for the Pocket Ratings backend with strict Clippy settings. Use when verifying backend code quality, before submitting backend changes, or when the user asks to run QC, lint, or check that the backend passes.
---

# Backend Quality Control

Run these checks whenever you need to verify the Pocket Ratings backend passes quality control. Use this skill when confirming backend changes are ready, before marking work done, or when the user asks to run QC, lint, or check the backend.

**Backend work must not be marked complete until all checks in this skill (format, Clippy, tests, and coverage) have been run and passed.** Running only a subset (e.g. only tests or only coverage) is not sufficient. See the **task-completion-qa** rule.

**If `cargo` fails with "rustup could not choose a version" or "no default is configured":** run `rustup default stable` (with network permission) first, then retry. Do not treat it as a code bug.

## Order of checks

Run in this order from the project root (or use `cd backend` first and then the commands without the `cd` prefix).

### 1. Format

```bash
cd backend && cargo fmt
```

Ensures formatting is consistent. Fix any reformatting by re-running after code changes.

### 2. Lint (strict pedantic Clippy)

Run Clippy on **all targets** (lib, bin, tests, examples):

```bash
cd backend && cargo clippy --all-targets --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings
```

All warnings are treated as errors.

If Clippy reports fixable issues, you can use `--fix` to apply suggestions automatically, then re-run without `--fix` to confirm a clean run:

```bash
cd backend

# Apply automatic fixes (may need multiple iterations)
cargo clippy --all-targets --fix -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings

# When fixes are applied, verify lint is clean
cargo clippy --all-targets --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings
```

### 3. Test

```bash
cd backend && cargo test --release -- --skip server_start_and_stop_via_cli
```

All tests must pass. Skip the long-running `server_start_and_stop_via_cli` test (it starts/stops the server and can exceed timeouts).

## One-shot (all checks)

```bash
cd backend && cargo fmt && cargo clippy --all-targets --release -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo -D warnings && cargo test --release -- --skip server_start_and_stop_via_cli
```

### 4. Coverage

CI and pre-push enforce a line-coverage threshold via
`./scripts/backend-coverage.sh` (run from repo root). To run coverage
locally with the same threshold:

```bash
./scripts/backend-coverage.sh
```

Requires `cargo-llvm-cov` (`cargo install cargo-llvm-cov`). The script runs
tests with `--skip server_start_and_stop_via_cli` and fails if line
coverage is below the threshold.

To see coverage without failing on threshold (e.g. to inspect the summary):

```bash
cd backend && cargo llvm-cov -- --skip server_start_and_stop_via_cli
```

To generate LCOV for the coverage-report script or other tools:

```bash
cd backend && cargo llvm-cov --lcov --output-path lcov.info -- --skip server_start_and_stop_via_cli
```

To generate an HTML report (opens in browser with `--open`):

```bash
cd backend && cargo llvm-cov --html -- --skip server_start_and_stop_via_cli
```

To find which files need more coverage and add tests, use the
**backend-coverage-improvement** skill and `scripts/backend-coverage-report.py`.

## Done when

- `cargo fmt` has been run and the tree is formatted.
- `cargo clippy` with the flags above exits 0 (no warnings, no errors) for all targets (including tests).
- `cargo test --release -- --skip server_start_and_stop_via_cli` exits 0 (all tests pass, long-running server test skipped).
- `./scripts/backend-coverage.sh` exits 0 (line coverage meets threshold).
