---
name: backend-quality-control
description: Run format, lint, and test for the Pocket Ratings backend with strict Clippy settings. Use when verifying backend code quality, before submitting backend changes, or when the user asks to run QC, lint, or check that the backend passes.
---

# Backend Quality Control

Run these checks whenever you need to verify the Pocket Ratings backend passes quality control. Use this skill when confirming backend changes are ready, before marking work done, or when the user asks to run QC, lint, or check the backend.

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

## Done when

- `cargo fmt` has been run and the tree is formatted.
- `cargo clippy` with the flags above exits 0 (no warnings, no errors) for all targets (including tests).
- `cargo test --release -- --skip server_start_and_stop_via_cli` exits 0 (all tests pass, long-running server test skipped).
